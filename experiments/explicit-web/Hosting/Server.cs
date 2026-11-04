using System.Net;
using ExplicitWeb;
using Microsoft.AspNetCore.Hosting.Server;
using Microsoft.AspNetCore.Hosting.Server.Features;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Http.Features;
using BadHttpRequestException = Microsoft.AspNetCore.Http.BadHttpRequestException;
using Microsoft.AspNetCore.Server.Kestrel.Core;
using Microsoft.AspNetCore.Server.Kestrel.Transport.Sockets;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;

namespace ExplicitWeb.Hosting;

public sealed record ServerSettings(IPAddress Address, int Port, int MaxBodyBytes, TimeSpan RequestTimeout)
{
    public void Validate()
    {
        if (Port is < 0 or > 65535 || MaxBodyBytes is < 1 or > 16_777_216 || RequestTimeout <= TimeSpan.Zero)
            throw new ArgumentException("Configure a valid port, a body limit up to 16 MiB, and a positive timeout.");
    }
}
public static class Server
{
    public static async Task Run(ServerSettings settings, Func<Request, Task<Cors.Prepared>> handle,
        Action<Uri> ready, CancellationToken stop)
    {
        settings.Validate();
        using var logger = LoggerFactory.Create(builder => builder.AddJsonConsole());
        var options = new KestrelServerOptions();
        options.Listen(settings.Address, settings.Port);
        options.Limits.MaxRequestBodySize = settings.MaxBodyBytes;
        options.Limits.RequestHeadersTimeout = settings.RequestTimeout;
        options.AddServerHeader = false;
        using var kestrel = new KestrelServer(Options.Create(options),
            new SocketTransportFactory(Options.Create(new SocketTransportOptions()), logger), logger);
        await kestrel.StartAsync(new Application(settings, handle, logger.CreateLogger("ExplicitWeb")), stop);
        ready(new Uri(kestrel.Features.Get<IServerAddressesFeature>()!.Addresses.Single()));
        try { await Task.Delay(Timeout.InfiniteTimeSpan, stop); }
        catch (OperationCanceledException) when (stop.IsCancellationRequested) { }
        finally
        {
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(10));
            await kestrel.StopAsync(timeout.Token);
        }
    }
}
internal sealed class Application(ServerSettings settings, Func<Request, Task<Cors.Prepared>> handle, ILogger logger) : IHttpApplication<DefaultHttpContext>
{
    private static readonly Cors FallbackCors = new([], [], []);
    public DefaultHttpContext CreateContext(IFeatureCollection features) => new(features);
    public void DisposeContext(DefaultHttpContext context, Exception? exception) { }
    public async Task ProcessRequestAsync(DefaultHttpContext context)
    {
        var started = System.Diagnostics.Stopwatch.StartNew();
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(context.RequestAborted);
        deadline.CancelAfter(settings.RequestTimeout);
        var input = context.Request;
        var localOrigin = new UriBuilder("http", context.Connection.LocalIpAddress!.ToString(), context.Connection.LocalPort).Uri.GetLeftPart(UriPartial.Authority);
        var request = new Request(input.Method, input.Path.Value ?? "/", context.Connection.RemoteIpAddress?.ToString() ?? "unknown",
            input.Headers.ToDictionary(x => x.Key, x => x.Value.ToString()), [], deadline.Token, localOrigin);
        Cors.Prepared prepared;
        try
        {
            using var body = new MemoryStream();
            var buffer = new byte[8192];
            while (true)
            {
                int count = await input.Body.ReadAsync(buffer, deadline.Token);
                if (count == 0) break;
                if (body.Length + count > settings.MaxBodyBytes) throw new BadHttpRequestException("Request body exceeds the configured limit.", 413);
                body.Write(buffer, 0, count);
            }
            request = new Request(input.Method, input.Path.Value ?? "/", request.Peer,
                input.Headers.ToDictionary(x => x.Key, x => x.Value.ToString()), body.ToArray(), deadline.Token, localOrigin);
            prepared = await handle(request);
        }
        catch (BadHttpRequestException e)
        { prepared = FallbackCors.Inspect(request).Apply(Response.Text("Request rejected by the HTTP transport. Check the request format and body limit.", e.StatusCode)); }
        catch (OperationCanceledException) when (deadline.IsCancellationRequested)
        { prepared = FallbackCors.Inspect(request).Apply(Response.Text("Request timed out. Try again.", 408)); }
        catch (Exception e)
        {
            logger.LogError(e, "Unhandled request {RequestId}", request.Id);
            prepared = FallbackCors.Inspect(request).Apply(Response.Text($"Request failed. Contact the operator with request ID {request.Id}.", 500));
        }
        var response = prepared.Response;
        context.Response.StatusCode = response.Status;
        context.Response.ContentType = response.ContentType;
        foreach (var header in response.Headers) context.Response.Headers[header.Key] = header.Value;
        if (response.Status is not (204 or 304)) context.Response.ContentLength = response.Body.Length;
        if (request.Method != "HEAD" && response.Status is not (204 or 304))
            await context.Response.Body.WriteAsync(response.Body, context.RequestAborted);
        logger.LogInformation("Request {RequestId} {Method} {Path} returned {Status} in {Milliseconds} ms",
            request.Id, request.Method, request.Path, response.Status, started.Elapsed.TotalMilliseconds);
    }
}
