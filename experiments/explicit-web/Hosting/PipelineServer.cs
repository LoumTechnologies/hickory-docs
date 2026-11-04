using ExplicitWeb;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Hosting.Server;
using Microsoft.AspNetCore.Hosting.Server.Features;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

namespace ExplicitWeb.Hosting;

public static class PipelineServer
{
    public static async Task Run(ServerSettings settings, Func<Request, Task<Cors.Prepared>> handle, Action<Uri> ready, CancellationToken stop)
    {
        settings.Validate();
        using var logger = LoggerFactory.Create(builder => builder.AddJsonConsole());
        var adapter = new Application(settings, handle, logger.CreateLogger("ExplicitWeb"));
        using var host = new HostBuilder().ConfigureWebHost(web => web
            .UseKestrel(options => { options.Listen(settings.Address, settings.Port); options.Limits.MaxRequestBodySize = settings.MaxBodyBytes; options.AddServerHeader = false; options.Limits.RequestHeadersTimeout = settings.RequestTimeout; })
            .Configure(app => app.Run(context => adapter.ProcessRequestAsync((DefaultHttpContext)context)))).Build();
        await host.StartAsync(stop);
        ready(new Uri(host.Services.GetService(typeof(IServer)) is IServer server
            ? server.Features.Get<IServerAddressesFeature>()!.Addresses.Single() : throw new InvalidOperationException("Host did not expose a server.")));
        try { await host.WaitForShutdownAsync(stop); }
        catch (OperationCanceledException) when (stop.IsCancellationRequested) { }
    }
}
