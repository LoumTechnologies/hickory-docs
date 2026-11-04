using System.Runtime.InteropServices;
using ExplicitWeb;
using ExplicitWeb.Data;
using ExplicitWeb.Hosting;
using ExplicitWeb.Security;
namespace TaskApp;

public static class Bootstrap
{
public static async Task Run(bool pipeline)
{
var settings = Settings.Load();
var database = new Database(settings.Database);
Schema.Initialize(database);
var clock = TimeProvider.System;
var sessions = new Sessions(database, clock, settings.SessionLifetime, settings.SecureCookie);
var accounts = new Accounts(database, sessions);
var cors = new Cors([settings.Origin], ["GET", "POST", "PUT", "DELETE"], ["content-type", "x-csrf-token"]);
var application = new TaskApp.Application(cors, sessions, accounts, new Tasks(database), new RateLimit(clock, settings.LoginAttempts, TimeSpan.FromMinutes(1)));
using var stop = new CancellationTokenSource();
Console.CancelKeyPress += (_, e) => { e.Cancel = true; stop.Cancel(); };
using var terminate = OperatingSystem.IsWindows() ? null : PosixSignalRegistration.Create(PosixSignal.SIGTERM, context => { context.Cancel = true; stop.Cancel(); });
try
{
    Func<ExplicitWeb.Request, Task<Cors.Prepared>> handle = request => Task.FromResult(application.Handle(request));
    Action<Uri> ready = uri => File.WriteAllText(settings.ReadyFile, uri.GetLeftPart(UriPartial.Authority));
    if (pipeline) await PipelineServer.Run(settings.Server, handle, ready, stop.Token);
    else await Server.Run(settings.Server, handle, ready, stop.Token);
}
finally { File.Delete(settings.ReadyFile); }
}
}
