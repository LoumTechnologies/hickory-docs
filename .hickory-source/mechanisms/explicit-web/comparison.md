<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Hold the application constant while changing its host

For evaluators measuring transport overhead. The conventional arm uses ASP.NET Core’s normal terminal request delegate. Both arms run the identical application, checked values, security, persistence and serialization. This isolates hosting overhead; it cannot estimate the productivity or performance cost of the type-oriented API versus a separately written MVC application.

## Pipeline control

Use the standard host and an ordinary terminal delegate as a control. Share the bounded request/response adapter so changing body limits, error handling or serialization does not confound the comparison. There are no extra artificial middleware layers to make the control slower.

<web:copy id="ew-pipeline-control">
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
</web:copy>

## Baseline baseline csproj

Keep assembly boundaries real: internal constructors cannot be called by consumers. Nullable warnings are errors; deterministic builds and dependency locks make evaluation reproducible.

<web:copy id="ew-Baseline-Baseline-csproj">
<Project Sdk="Microsoft.NET.Sdk.Web">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><ImplicitUsings>enable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors><UseAppHost>false</UseAppHost>
    <RestorePackagesWithLockFile>true</RestorePackagesWithLockFile><Deterministic>true</Deterministic>
  </PropertyGroup>
  <ItemGroup><ProjectReference Include="../Server/Server.csproj" /></ItemGroup>
</Project>
</web:copy>

## Baseline entry

The control chooses the conventional host and shares everything else. HTTP acceptance tests select this executable via their launcher setting.

<web:copy id="ew-baseline-entry">
using TaskApp;
await Bootstrap.Run(true);
</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/Hosting/PipelineServer.cs"><web:paste select="#ew-pipeline-control"/></web:file>
<web:file path="experiments/explicit-web/Baseline/Baseline.csproj"><web:paste select="#ew-Baseline-Baseline-csproj"/></web:file>
<web:file path="experiments/explicit-web/Baseline/Program.cs"><web:paste select="#ew-baseline-entry"/></web:file>

</web:doc>
