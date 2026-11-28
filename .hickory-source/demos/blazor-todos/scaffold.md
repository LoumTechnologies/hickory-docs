<demo:doc xmlns:demo="http://www.hickorydocs.com/1.0">
# Build a signed-in Blazor todo app

For someone trying Hickory's New Project workflow and following the result from
a generated app to a new model and its UI. The runnable .NET project lives in
`examples/projects/blazor-todos/app`. Open this document in Hickory from the
repository root to inspect the scaffold and its changes.

## Capture an authenticated server app

In the macOS app, choose **File > New > .NET Project**, then **Blazor Web App
(C#)**. Select **Server** interactivity and **Individual** authentication.
Use the empty template and leave global interactivity off. Identity keeps its
account pages on the server request path, where it can issue cookies. The
capture command below selects those same options and pins the SDK family.
It also skips restore so the scaffold capture contains source only.

<demo:container name="sdk" image="mcr.microsoft.com/dotnet/sdk:10.0" />
<demo:volume name="blazor" output="examples/projects/blazor-todos" />
<demo:exec id="blazor-scaffold" container="sdk" mount="blazor:out">
<demo:copy id="dotnet-new-blazor">
set -eu
export DOTNET_NOLOGO=1 DOTNET_CLI_TELEMETRY_OPTOUT=1
export DOTNET_GENERATE_ASPNET_CERTIFICATE=false
cd out
cat > global.json <<'SDK'
{"sdk":{"version":"10.0.100","rollForward":"latestFeature","allowPrerelease":false}}
SDK
dotnet new blazor --name HickoryTodos --framework net10.0 --interactivity Server --auth Individual --empty --no-restore --no-update-check --output app
</demo:copy>
<demo:ingested from="#dotnet-new-blazor" sha256="267630fc3c36c420d3081367cb22a0e5542cdea019fa3f2c7d5977add4529b03" at="2026-11-28" files="70" skipped="2">
<demo:file path="examples/projects/blazor-todos/app/Components/Account/IdentityComponentsEndpointRouteBuilderExtensions.cs">
using System.Security.Claims;
using System.Text.Json;
using Microsoft.AspNetCore.Antiforgery;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Components.Authorization;
using Microsoft.AspNetCore.Http.Extensions;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Primitives;
using HickoryTodos.Components.Account.Pages;
using HickoryTodos.Components.Account.Pages.Manage;
using HickoryTodos.Data;

namespace Microsoft.AspNetCore.Routing;

internal static class IdentityComponentsEndpointRouteBuilderExtensions
{
    // These endpoints are required by the Identity Razor components defined in the /Components/Account/Pages directory of this project.
    public static IEndpointConventionBuilder MapAdditionalIdentityEndpoints(this IEndpointRouteBuilder endpoints)
    {
        ArgumentNullException.ThrowIfNull(endpoints);

        var accountGroup = endpoints.MapGroup("/Account");

        accountGroup.MapPost("/PerformExternalLogin", (
            HttpContext context,
            [FromServices] SignInManager<ApplicationUser> signInManager,
            [FromForm] string provider,
            [FromForm] string returnUrl) =>
        {
            IEnumerable<KeyValuePair<string, StringValues>> query = [
                new("ReturnUrl", returnUrl),
                new("Action", ExternalLogin.LoginCallbackAction)];

            var redirectUrl = UriHelper.BuildRelative(
                context.Request.PathBase,
                "/Account/ExternalLogin",
                QueryString.Create(query));

            var properties = signInManager.ConfigureExternalAuthenticationProperties(provider, redirectUrl);
            return TypedResults.Challenge(properties, [provider]);
        });

        accountGroup.MapPost("/Logout", async (
            ClaimsPrincipal user,
            [FromServices] SignInManager<ApplicationUser> signInManager,
            [FromForm] string returnUrl) =>
        {
            await signInManager.SignOutAsync();
            return TypedResults.LocalRedirect($"~/{returnUrl}");
        });

        accountGroup.MapPost("/PasskeyCreationOptions", async (
            HttpContext context,
            [FromServices] UserManager<ApplicationUser> userManager,
            [FromServices] SignInManager<ApplicationUser> signInManager,
            [FromServices] IAntiforgery antiforgery) =>
        {
            await antiforgery.ValidateRequestAsync(context);

            var user = await userManager.GetUserAsync(context.User);
            if (user is null)
            {
                return Results.NotFound($"Unable to load user with ID '{userManager.GetUserId(context.User)}'.");
            }

            var userId = await userManager.GetUserIdAsync(user);
            var userName = await userManager.GetUserNameAsync(user) ?? "User";
            var optionsJson = await signInManager.MakePasskeyCreationOptionsAsync(new()
            {
                Id = userId,
                Name = userName,
                DisplayName = userName
            });
            return TypedResults.Content(optionsJson, contentType: "application/json");
        });

        accountGroup.MapPost("/PasskeyRequestOptions", async (
            HttpContext context,
            [FromServices] UserManager<ApplicationUser> userManager,
            [FromServices] SignInManager<ApplicationUser> signInManager,
            [FromServices] IAntiforgery antiforgery,
            [FromQuery] string? username) =>
        {
            await antiforgery.ValidateRequestAsync(context);

            var user = string.IsNullOrEmpty(username) ? null : await userManager.FindByNameAsync(username);
            var optionsJson = await signInManager.MakePasskeyRequestOptionsAsync(user);
            return TypedResults.Content(optionsJson, contentType: "application/json");
        });

        var manageGroup = accountGroup.MapGroup("/Manage").RequireAuthorization();

        manageGroup.MapPost("/LinkExternalLogin", async (
            HttpContext context,
            [FromServices] SignInManager<ApplicationUser> signInManager,
            [FromForm] string provider) =>
        {
            // Clear the existing external cookie to ensure a clean login process
            await context.SignOutAsync(IdentityConstants.ExternalScheme);

            var redirectUrl = UriHelper.BuildRelative(
                context.Request.PathBase,
                "/Account/Manage/ExternalLogins",
                QueryString.Create("Action", ExternalLogins.LinkLoginCallbackAction));

            var properties = signInManager.ConfigureExternalAuthenticationProperties(provider, redirectUrl, signInManager.UserManager.GetUserId(context.User));
            return TypedResults.Challenge(properties, [provider]);
        });

        var loggerFactory = endpoints.ServiceProvider.GetRequiredService<ILoggerFactory>();
        var downloadLogger = loggerFactory.CreateLogger("DownloadPersonalData");

        manageGroup.MapPost("/DownloadPersonalData", async (
            HttpContext context,
            [FromServices] UserManager<ApplicationUser> userManager,
            [FromServices] AuthenticationStateProvider authenticationStateProvider) =>
        {
            var user = await userManager.GetUserAsync(context.User);
            if (user is null)
            {
                return Results.NotFound($"Unable to load user with ID '{userManager.GetUserId(context.User)}'.");
            }

            var userId = await userManager.GetUserIdAsync(user);
            downloadLogger.LogInformation("User with ID '{UserId}' asked for their personal data.", userId);

            // Only include personal data for download
            var personalData = new Dictionary<string, string>();
            var personalDataProps = typeof(ApplicationUser).GetProperties().Where(
                prop => Attribute.IsDefined(prop, typeof(PersonalDataAttribute)));
            foreach (var p in personalDataProps)
            {
                personalData.Add(p.Name, p.GetValue(user)?.ToString() ?? "null");
            }

            var logins = await userManager.GetLoginsAsync(user);
            foreach (var l in logins)
            {
                personalData.Add($"{l.LoginProvider} external login provider key", l.ProviderKey);
            }

            personalData.Add("Authenticator Key", (await userManager.GetAuthenticatorKeyAsync(user))!);
            var fileBytes = JsonSerializer.SerializeToUtf8Bytes(personalData);

            context.Response.Headers.TryAdd("Content-Disposition", "attachment; filename=PersonalData.json");
            return TypedResults.File(fileBytes, contentType: "application/json", fileDownloadName: "PersonalData.json");
        });

        return accountGroup;
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/IdentityNoOpEmailSender.cs">
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Identity.UI.Services;
using HickoryTodos.Data;

namespace HickoryTodos.Components.Account;

// Remove the "else if (EmailSender is IdentityNoOpEmailSender)" block from RegisterConfirmation.razor after updating with a real implementation.
internal sealed class IdentityNoOpEmailSender : IEmailSender<ApplicationUser>
{
    private readonly IEmailSender emailSender = new NoOpEmailSender();

    public Task SendConfirmationLinkAsync(ApplicationUser user, string email, string confirmationLink) =>
        emailSender.SendEmailAsync(email, "Confirm your email", $"Please confirm your account by <a href='{confirmationLink}'>clicking here</a>.");

    public Task SendPasswordResetLinkAsync(ApplicationUser user, string email, string resetLink) =>
        emailSender.SendEmailAsync(email, "Reset your password", $"Please reset your password by <a href='{resetLink}'>clicking here</a>.");

    public Task SendPasswordResetCodeAsync(ApplicationUser user, string email, string resetCode) =>
        emailSender.SendEmailAsync(email, "Reset your password", $"Please reset your password using the following code: {resetCode}");
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/IdentityRedirectManager.cs">
using Microsoft.AspNetCore.Components;
using Microsoft.AspNetCore.Identity;
using HickoryTodos.Data;

namespace HickoryTodos.Components.Account;

internal sealed class IdentityRedirectManager(NavigationManager navigationManager)
{
    public const string StatusCookieName = "Identity.StatusMessage";

    private static readonly CookieBuilder StatusCookieBuilder = new()
    {
        SameSite = SameSiteMode.Strict,
        HttpOnly = true,
        IsEssential = true,
        MaxAge = TimeSpan.FromSeconds(5),
    };

    public void RedirectTo(string? uri)
    {
        uri ??= "";

        // Prevent open redirects.
        if (!Uri.IsWellFormedUriString(uri, UriKind.Relative)
            || uri.StartsWith("//", StringComparison.Ordinal))
        {
            uri = navigationManager.ToBaseRelativePath(uri);
        }

        navigationManager.NavigateTo(uri);
    }

    public void RedirectTo(string uri, Dictionary<string, object?> queryParameters)
    {
        var uriWithoutQuery = navigationManager.ToAbsoluteUri(uri).GetLeftPart(UriPartial.Path);
        var newUri = navigationManager.GetUriWithQueryParameters(uriWithoutQuery, queryParameters);
        RedirectTo(newUri);
    }

    public void RedirectToWithStatus(string uri, string message, HttpContext context)
    {
        context.Response.Cookies.Append(StatusCookieName, message, StatusCookieBuilder.Build(context));
        RedirectTo(uri);
    }

    private string CurrentPath => navigationManager.ToAbsoluteUri(navigationManager.Uri).GetLeftPart(UriPartial.Path);

    public void RedirectToCurrentPage() => RedirectTo(CurrentPath);

    public void RedirectToCurrentPageWithStatus(string message, HttpContext context)
        => RedirectToWithStatus(CurrentPath, message, context);

    public void RedirectToInvalidUser(UserManager<ApplicationUser> userManager, HttpContext context)
        => RedirectToWithStatus("Account/InvalidUser", $"Error: Unable to load user with ID '{userManager.GetUserId(context.User)}'.", context);
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/IdentityRevalidatingAuthenticationStateProvider.cs">
using System.Security.Claims;
using Microsoft.AspNetCore.Components.Authorization;
using Microsoft.AspNetCore.Components.Server;
using Microsoft.AspNetCore.Identity;
using Microsoft.Extensions.Options;
using HickoryTodos.Data;

namespace HickoryTodos.Components.Account;

// This is a server-side AuthenticationStateProvider that revalidates the security stamp for the connected user
// every 30 minutes an interactive circuit is connected.
internal sealed class IdentityRevalidatingAuthenticationStateProvider(
        ILoggerFactory loggerFactory,
        IServiceScopeFactory scopeFactory,
        IOptions<IdentityOptions> options)
    : RevalidatingServerAuthenticationStateProvider(loggerFactory)
{
    protected override TimeSpan RevalidationInterval => TimeSpan.FromMinutes(30);

    protected override async Task<bool> ValidateAuthenticationStateAsync(
        AuthenticationState authenticationState, CancellationToken cancellationToken)
    {
        // Get the user manager from a new scope to ensure it fetches fresh data
        await using var scope = scopeFactory.CreateAsyncScope();
        var userManager = scope.ServiceProvider.GetRequiredService<UserManager<ApplicationUser>>();
        return await ValidateSecurityStampAsync(userManager, authenticationState.User);
    }

    private async Task<bool> ValidateSecurityStampAsync(UserManager<ApplicationUser> userManager, ClaimsPrincipal principal)
    {
        var user = await userManager.GetUserAsync(principal);
        if (user is null)
        {
            return false;
        }
        else if (!userManager.SupportsUserSecurityStamp)
        {
            return true;
        }
        else
        {
            var principalStamp = principal.FindFirstValue(options.Value.ClaimsIdentity.SecurityStampClaimType);
            var userStamp = await userManager.GetSecurityStampAsync(user);
            return principalStamp == userStamp;
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/AccessDenied.razor">
﻿@page "/Account/AccessDenied"

<PageTitle>Access denied</PageTitle>

<header>
    <h1 class="text-danger">Access denied</h1>
    <p class="text-danger">You do not have access to this resource.</p>
</header>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/ConfirmEmail.razor">
﻿@page "/Account/ConfirmEmail"

@using System.Text
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Confirm email</PageTitle>

<h1>Confirm email</h1>
<StatusMessage Message="@statusMessage" />

@code {
    private string? statusMessage;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? UserId { get; set; }

    [SupplyParameterFromQuery]
    private string? Code { get; set; }

    protected override async Task OnInitializedAsync()
    {
        if (UserId is null || Code is null)
        {
            RedirectManager.RedirectTo("");
            return;
        }

        var user = await UserManager.FindByIdAsync(UserId);
        if (user is null)
        {
            HttpContext.Response.StatusCode = StatusCodes.Status404NotFound;
            statusMessage = $"Error loading user with ID {UserId}";
        }
        else
        {
            var code = Encoding.UTF8.GetString(WebEncoders.Base64UrlDecode(Code));
            var result = await UserManager.ConfirmEmailAsync(user, code);
            statusMessage = result.Succeeded ? "Thank you for confirming your email." : "Error confirming your email.";
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/ConfirmEmailChange.razor">
﻿@page "/Account/ConfirmEmailChange"

@using System.Text
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Confirm email change</PageTitle>

<h1>Confirm email change</h1>

<StatusMessage Message="@message" />

@code {
    private string? message;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? UserId { get; set; }

    [SupplyParameterFromQuery]
    private string? Email { get; set; }

    [SupplyParameterFromQuery]
    private string? Code { get; set; }

    protected override async Task OnInitializedAsync()
    {
        if (UserId is null || Email is null || Code is null)
        {
            RedirectManager.RedirectToWithStatus(
                "Account/Login", "Error: Invalid email change confirmation link.", HttpContext);
            return;
        }

        var user = await UserManager.FindByIdAsync(UserId);
        if (user is null)
        {
            message = "Unable to find user with Id '{userId}'";
            return;
        }

        var code = Encoding.UTF8.GetString(WebEncoders.Base64UrlDecode(Code));
        var result = await UserManager.ChangeEmailAsync(user, Email, code);
        if (!result.Succeeded)
        {
            message = "Error changing email.";
            return;
        }

        // In our UI email and user name are one and the same, so when we update the email
        // we need to update the user name.
        var setUserNameResult = await UserManager.SetUserNameAsync(user, Email);
        if (!setUserNameResult.Succeeded)
        {
            message = "Error changing user name.";
            return;
        }

        await SignInManager.RefreshSignInAsync(user);
        message = "Thank you for confirming your email change.";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/ExternalLogin.razor">
﻿@page "/Account/ExternalLogin"

@using System.ComponentModel.DataAnnotations
@using System.Security.Claims
@using System.Text
@using System.Text.Encodings.Web
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject SignInManager<ApplicationUser> SignInManager
@inject UserManager<ApplicationUser> UserManager
@inject IUserStore<ApplicationUser> UserStore
@inject IEmailSender<ApplicationUser> EmailSender
@inject NavigationManager NavigationManager
@inject IdentityRedirectManager RedirectManager
@inject ILogger<ExternalLogin> Logger

<PageTitle>Register</PageTitle>

<StatusMessage Message="@message" />
<h1>Register</h1>
<h2>Associate your @ProviderDisplayName account.</h2>
<hr />

<div  class="alert alert-info">
    You've successfully authenticated with <strong>@ProviderDisplayName</strong>.
    Please enter an email address for this site below and click the Register button to finish
    logging in.
</div>

<div class="row">
    <div class="col-md-4">
        <EditForm Model="Input" OnValidSubmit="OnValidSubmitAsync" FormName="confirmation" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />
            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.Email" id="Input.Email" class="form-control" autocomplete="email" placeholder="Please enter your email." />
                <label for="Input.Email" class="form-label">Email</label>
                <ValidationMessage For="() => Input.Email" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Register</button>
        </EditForm>
    </div>
</div>

@code {
    public const string LoginCallbackAction = "LoginCallback";

    private string? message;
    private ExternalLoginInfo? externalLoginInfo;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? RemoteError { get; set; }

    [SupplyParameterFromQuery]
    private string? ReturnUrl { get; set; }

    [SupplyParameterFromQuery]
    private string? Action { get; set; }

    private string? ProviderDisplayName => externalLoginInfo?.ProviderDisplayName;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        if (RemoteError is not null)
        {
            RedirectManager.RedirectToWithStatus("Account/Login", $"Error from external provider: {RemoteError}", HttpContext);
            return;
        }

        var info = await SignInManager.GetExternalLoginInfoAsync();
        if (info is null)
        {
            RedirectManager.RedirectToWithStatus("Account/Login", "Error loading external login information.", HttpContext);
            return;
        }

        externalLoginInfo = info;

        if (HttpMethods.IsGet(HttpContext.Request.Method))
        {
            if (Action == LoginCallbackAction)
            {
                await OnLoginCallbackAsync();
                return;
            }

            // We should only reach this page via the login callback, so redirect back to
            // the login page if we get here some other way.
            RedirectManager.RedirectTo("Account/Login");
        }
    }

    private async Task OnLoginCallbackAsync()
    {
        if (externalLoginInfo is null)
        {
            RedirectManager.RedirectToWithStatus("Account/Login", "Error loading external login information.", HttpContext);
            return;
        }

        // Sign in the user with this external login provider if the user already has a login.
        var result = await SignInManager.ExternalLoginSignInAsync(
            externalLoginInfo.LoginProvider,
            externalLoginInfo.ProviderKey,
            isPersistent: false,
            bypassTwoFactor: true);

        if (result.Succeeded)
        {
            Logger.LogInformation(
                "{Name} logged in with {LoginProvider} provider.",
                externalLoginInfo.Principal.Identity?.Name,
                externalLoginInfo.LoginProvider);
            RedirectManager.RedirectTo(ReturnUrl);
            return;
        }
        else if (result.IsLockedOut)
        {
            RedirectManager.RedirectTo("Account/Lockout");
            return;
        }

        // If the user does not have an account, then ask the user to create an account.
        if (externalLoginInfo.Principal.HasClaim(c => c.Type == ClaimTypes.Email))
        {
            Input.Email = externalLoginInfo.Principal.FindFirstValue(ClaimTypes.Email) ?? "";
        }
    }

    private async Task OnValidSubmitAsync()
    {
        if (externalLoginInfo is null)
        {
            RedirectManager.RedirectToWithStatus("Account/Login", "Error loading external login information during confirmation.", HttpContext);
            return;
        }

        var emailStore = GetEmailStore();
        var user = CreateUser();

        await UserStore.SetUserNameAsync(user, Input.Email, CancellationToken.None);
        await emailStore.SetEmailAsync(user, Input.Email, CancellationToken.None);

        var result = await UserManager.CreateAsync(user);
        if (result.Succeeded)
        {
            result = await UserManager.AddLoginAsync(user, externalLoginInfo);
            if (result.Succeeded)
            {
                Logger.LogInformation("User created an account using {Name} provider.", externalLoginInfo.LoginProvider);

                var userId = await UserManager.GetUserIdAsync(user);
                var code = await UserManager.GenerateEmailConfirmationTokenAsync(user);
                code = WebEncoders.Base64UrlEncode(Encoding.UTF8.GetBytes(code));

                var callbackUrl = NavigationManager.GetUriWithQueryParameters(
                    NavigationManager.ToAbsoluteUri("Account/ConfirmEmail").AbsoluteUri,
                    new Dictionary<string, object?> { ["userId"] = userId, ["code"] = code });
                await EmailSender.SendConfirmationLinkAsync(user, Input.Email, HtmlEncoder.Default.Encode(callbackUrl));

                // If account confirmation is required, we need to show the link if we don't have a real email sender
                if (UserManager.Options.SignIn.RequireConfirmedAccount)
                {
                    RedirectManager.RedirectTo("Account/RegisterConfirmation", new() { ["email"] = Input.Email });
                }
                else
                {
                    await SignInManager.SignInAsync(user, isPersistent: false, externalLoginInfo.LoginProvider);
                    RedirectManager.RedirectTo(ReturnUrl);
                }
            }
        }
        else
        {
            message = $"Error: {string.Join(",", result.Errors.Select(error => error.Description))}";
        }
    }

    private ApplicationUser CreateUser()
    {
        try
        {
            return Activator.CreateInstance<ApplicationUser>();
        }
        catch
        {
            throw new InvalidOperationException($"Can't create an instance of '{nameof(ApplicationUser)}'. " +
                $"Ensure that '{nameof(ApplicationUser)}' is not an abstract class and has a parameterless constructor");
        }
    }

    private IUserEmailStore<ApplicationUser> GetEmailStore()
    {
        if (!UserManager.SupportsUserEmail)
        {
            throw new NotSupportedException("The default UI requires a user store with email support.");
        }
        return (IUserEmailStore<ApplicationUser>)UserStore;
    }

    private sealed class InputModel
    {
        [Required]
        [EmailAddress]
        public string Email { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/ForgotPassword.razor">
﻿@page "/Account/ForgotPassword"

@using System.ComponentModel.DataAnnotations
@using System.Text
@using System.Text.Encodings.Web
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IEmailSender<ApplicationUser> EmailSender
@inject NavigationManager NavigationManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Forgot your password?</PageTitle>

<h1>Forgot your password?</h1>
<h2>Enter your email.</h2>
<hr />
<div class="row">
    <div class="col-md-4">
        <EditForm Model="Input" FormName="forgot-password" OnValidSubmit="OnValidSubmitAsync" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />

            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.Email" id="Input.Email" class="form-control" autocomplete="username" aria-required="true" placeholder="name@example.com" />
                <label for="Input.Email" class="form-label">Email</label>
                <ValidationMessage For="() => Input.Email" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Reset password</button>
        </EditForm>
     </div>
</div>

@code {
    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    protected override void OnInitialized()
    {
        Input ??= new();
    }

    private async Task OnValidSubmitAsync()
    {
        var user = await UserManager.FindByEmailAsync(Input.Email);
        if (user is null || !(await UserManager.IsEmailConfirmedAsync(user)))
        {
            // Don't reveal that the user does not exist or is not confirmed
            RedirectManager.RedirectTo("Account/ForgotPasswordConfirmation");
            return;
        }

        // For more information on how to enable account confirmation and password reset please
        // visit https://go.microsoft.com/fwlink/?LinkID=532713
        var code = await UserManager.GeneratePasswordResetTokenAsync(user);
        code = WebEncoders.Base64UrlEncode(Encoding.UTF8.GetBytes(code));
        var callbackUrl = NavigationManager.GetUriWithQueryParameters(
            NavigationManager.ToAbsoluteUri("Account/ResetPassword").AbsoluteUri,
            new Dictionary<string, object?> { ["code"] = code });

        await EmailSender.SendPasswordResetLinkAsync(user, Input.Email, HtmlEncoder.Default.Encode(callbackUrl));

        RedirectManager.RedirectTo("Account/ForgotPasswordConfirmation");
    }

    private sealed class InputModel
    {
        [Required]
        [EmailAddress]
        public string Email { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/ForgotPasswordConfirmation.razor">
﻿@page "/Account/ForgotPasswordConfirmation"

<PageTitle>Forgot password confirmation</PageTitle>

<h1>Forgot password confirmation</h1>
<p role="alert">
    Please check your email to reset your password.
</p>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/InvalidPasswordReset.razor">
﻿@page "/Account/InvalidPasswordReset"

<PageTitle>Invalid password reset</PageTitle>

<h1>Invalid password reset</h1>
<p role="alert">
    The password reset link is invalid.
</p>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/InvalidUser.razor">
﻿@page "/Account/InvalidUser"

<PageTitle>Invalid user</PageTitle>

<h3>Invalid user</h3>

<StatusMessage />
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Lockout.razor">
﻿@page "/Account/Lockout"

<PageTitle>Locked out</PageTitle>

<header>
    <h1 class="text-danger">Locked out</h1>
    <p class="text-danger" role="alert">This account has been locked out, please try again later.</p>
</header>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Login.razor">
﻿@page "/Account/Login"

@using System.ComponentModel.DataAnnotations
@using Microsoft.AspNetCore.Authentication
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject ILogger<Login> Logger
@inject NavigationManager NavigationManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Log in</PageTitle>

<h1>Log in</h1>
<div class="row">
    <div class="col-lg-6">
        <section>
            <StatusMessage Message="@errorMessage" />
            <EditForm EditContext="editContext" method="post" OnSubmit="LoginUser" FormName="login">
                <DataAnnotationsValidator />
                <h2>Use a local account to log in.</h2>
                <hr />
                <ValidationSummary class="text-danger" role="alert" />
                <div class="form-floating mb-3">
                    <InputText @bind-Value="Input.Email" id="Input.Email" class="form-control" autocomplete="username webauthn" aria-required="true" placeholder="name@example.com" />
                    <label for="Input.Email" class="form-label">Email</label>
                    <ValidationMessage For="() => Input.Email" class="text-danger" />
                </div>
                <div class="form-floating mb-3">
                    <InputText type="password" @bind-Value="Input.Password" id="Input.Password" class="form-control" autocomplete="current-password" aria-required="true" placeholder="password" />
                    <label for="Input.Password" class="form-label">Password</label>
                    <ValidationMessage For="() => Input.Password" class="text-danger" />
                </div>
                <div class="checkbox mb-3">
                    <label class="form-label">
                        <InputCheckbox @bind-Value="Input.RememberMe" class="darker-border-checkbox form-check-input" />
                        Remember me
                    </label>
                </div>
                <div>
                    <button type="submit" class="w-100 btn btn-lg btn-primary">Log in</button>
                </div>
                <hr />
                <div class="d-flex flex-column">
                    <span class="text-secondary mx-auto mt-2">OR</span>
                    <PasskeySubmit Operation="PasskeyOperation.Request" Name="Input.Passkey" EmailName="Input.Email" class="btn btn-link mx-auto">Log in with a passkey</PasskeySubmit>
                </div>
                <hr />
                <div>
                    <p>
                        <a href="Account/ForgotPassword">Forgot your password?</a>
                    </p>
                    <p>
                        <a href="@(NavigationManager.GetUriWithQueryParameters("Account/Register", new Dictionary<string, object?> { ["ReturnUrl"] = ReturnUrl }))">Register as a new user</a>
                    </p>
                    <p>
                        <a href="Account/ResendEmailConfirmation">Resend email confirmation</a>
                    </p>
                </div>
            </EditForm>
        </section>
    </div>
    <div class="col-lg-4 col-lg-offset-2">
        <section>
            <h3>Use another service to log in.</h3>
            <hr />
            <ExternalLoginPicker />
        </section>
    </div>
</div>

@code {
    private string? errorMessage;
    private EditContext editContext = default!;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? ReturnUrl { get; set; }

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        editContext = new EditContext(Input);

        if (HttpMethods.IsGet(HttpContext.Request.Method))
        {
            // Clear the existing external cookie to ensure a clean login process
            await HttpContext.SignOutAsync(IdentityConstants.ExternalScheme);
        }
    }

    public async Task LoginUser()
    {
        if (!string.IsNullOrEmpty(Input.Passkey?.Error))
        {
            errorMessage = $"Error: {Input.Passkey.Error}";
            return;
        }

        SignInResult result;
        if (!string.IsNullOrEmpty(Input.Passkey?.CredentialJson))
        {
            // When performing passkey sign-in, don't perform form validation.
            result = await SignInManager.PasskeySignInAsync(Input.Passkey.CredentialJson);
        }
        else
        {
            // If doing a password sign-in, validate the form.
            if (!editContext.Validate())
            {
                return;
            }

            // This doesn't count login failures towards account lockout
            // To enable password failures to trigger account lockout, set lockoutOnFailure: true
            result = await SignInManager.PasswordSignInAsync(Input.Email, Input.Password, Input.RememberMe, lockoutOnFailure: false);
        }

        if (result.Succeeded)
        {
            Logger.LogInformation("User logged in.");
            RedirectManager.RedirectTo(ReturnUrl);
        }
        else if (result.RequiresTwoFactor)
        {
            RedirectManager.RedirectTo(
                "Account/LoginWith2fa",
                new() { ["returnUrl"] = ReturnUrl, ["rememberMe"] = Input.RememberMe });
        }
        else if (result.IsLockedOut)
        {
            Logger.LogWarning("User account locked out.");
            RedirectManager.RedirectTo("Account/Lockout");
        }
        else
        {
            errorMessage = "Error: Invalid login attempt.";
        }
    }

    private sealed class InputModel
    {
        [Required]
        [EmailAddress]
        public string Email { get; set; } = "";

        [Required]
        [DataType(DataType.Password)]
        public string Password { get; set; } = "";

        [Display(Name = "Remember me?")]
        public bool RememberMe { get; set; }

        public PasskeyInputModel? Passkey { get; set; }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/LoginWith2fa.razor">
﻿@page "/Account/LoginWith2fa"

@using System.ComponentModel.DataAnnotations
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject SignInManager<ApplicationUser> SignInManager
@inject UserManager<ApplicationUser> UserManager
@inject IdentityRedirectManager RedirectManager
@inject ILogger<LoginWith2fa> Logger

<PageTitle>Two-factor authentication</PageTitle>

<h1>Two-factor authentication</h1>
<hr />
<StatusMessage Message="@message" />
<p>Your login is protected with an authenticator app. Enter your authenticator code below.</p>
<div class="row">
    <div class="col-md-4">
        <EditForm Model="Input" FormName="login-with-2fa" OnValidSubmit="OnValidSubmitAsync" method="post">
            <input type="hidden" name="ReturnUrl" value="@ReturnUrl" />
            <input type="hidden" name="RememberMe" value="@RememberMe" />
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />
            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.TwoFactorCode" id="Input.TwoFactorCode" class="form-control" autocomplete="off" />
                <label for="Input.TwoFactorCode" class="form-label">Authenticator code</label>
                <ValidationMessage For="() => Input.TwoFactorCode" class="text-danger" />
            </div>
            <div class="checkbox mb-3">
                <label for="remember-machine" class="form-label">
                    <InputCheckbox @bind-Value="Input.RememberMachine" />
                    Remember this machine
                </label>
            </div>
            <div>
                <button type="submit" class="w-100 btn btn-lg btn-primary">Log in</button>
            </div>
        </EditForm>
    </div>
</div>
<p>
    Don't have access to your authenticator device? You can
    <a href="Account/LoginWithRecoveryCode?ReturnUrl=@ReturnUrl">log in with a recovery code</a>.
</p>

@code {
    private string? message;
    private ApplicationUser user = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? ReturnUrl { get; set; }

    [SupplyParameterFromQuery]
    private bool RememberMe { get; set; }

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        // Ensure the user has gone through the username & password screen first
        user = await SignInManager.GetTwoFactorAuthenticationUserAsync() ??
            throw new InvalidOperationException("Unable to load two-factor authentication user.");
    }

    private async Task OnValidSubmitAsync()
    {
        var authenticatorCode = Input.TwoFactorCode!.Replace(" ", string.Empty).Replace("-", string.Empty);
        var result = await SignInManager.TwoFactorAuthenticatorSignInAsync(authenticatorCode, RememberMe, Input.RememberMachine);
        var userId = await UserManager.GetUserIdAsync(user);

        if (result.Succeeded)
        {
            Logger.LogInformation("User with ID '{UserId}' logged in with 2fa.", userId);
            RedirectManager.RedirectTo(ReturnUrl);
        }
        else if (result.IsLockedOut)
        {
            Logger.LogWarning("User with ID '{UserId}' account locked out.", userId);
            RedirectManager.RedirectTo("Account/Lockout");
        }
        else
        {
            Logger.LogWarning("Invalid authenticator code entered for user with ID '{UserId}'.", userId);
            message = "Error: Invalid authenticator code.";
        }
    }

    private sealed class InputModel
    {
        [Required]
        [StringLength(7, ErrorMessage = "The {0} must be at least {2} and at max {1} characters long.", MinimumLength = 6)]
        [DataType(DataType.Text)]
        [Display(Name = "Authenticator code")]
        public string? TwoFactorCode { get; set; }

        [Display(Name = "Remember this machine")]
        public bool RememberMachine { get; set; }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/LoginWithRecoveryCode.razor">
﻿@page "/Account/LoginWithRecoveryCode"

@using System.ComponentModel.DataAnnotations
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject SignInManager<ApplicationUser> SignInManager
@inject UserManager<ApplicationUser> UserManager
@inject IdentityRedirectManager RedirectManager
@inject ILogger<LoginWithRecoveryCode> Logger

<PageTitle>Recovery code verification</PageTitle>

<h1>Recovery code verification</h1>
<hr />
<StatusMessage Message="@message" />
<p>
    You have requested to log in with a recovery code. This login will not be remembered until you provide
    an authenticator app code at log in or disable 2FA and log in again.
</p>
<div class="row">
    <div class="col-md-4">
        <EditForm Model="Input" FormName="login-with-recovery-code" OnValidSubmit="OnValidSubmitAsync" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />
            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.RecoveryCode" id="Input.RecoveryCode" class="form-control" autocomplete="off" placeholder="RecoveryCode" />
                <label for="Input.RecoveryCode" class="form-label">Recovery Code</label>
                <ValidationMessage For="() => Input.RecoveryCode" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Log in</button>
        </EditForm>
    </div>
</div>

@code {
    private string? message;
    private ApplicationUser user = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? ReturnUrl { get; set; }

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        // Ensure the user has gone through the username & password screen first
        user = await SignInManager.GetTwoFactorAuthenticationUserAsync() ??
            throw new InvalidOperationException("Unable to load two-factor authentication user.");
    }

    private async Task OnValidSubmitAsync()
    {
        var recoveryCode = Input.RecoveryCode.Replace(" ", string.Empty);

        var result = await SignInManager.TwoFactorRecoveryCodeSignInAsync(recoveryCode);

        var userId = await UserManager.GetUserIdAsync(user);

        if (result.Succeeded)
        {
            Logger.LogInformation("User with ID '{UserId}' logged in with a recovery code.", userId);
            RedirectManager.RedirectTo(ReturnUrl);
        }
        else if (result.IsLockedOut)
        {
            Logger.LogWarning("User account locked out.");
            RedirectManager.RedirectTo("Account/Lockout");
        }
        else
        {
            Logger.LogWarning("Invalid recovery code entered for user with ID '{UserId}' ", userId);
            message = "Error: Invalid recovery code entered.";
        }
    }

    private sealed class InputModel
    {
        [Required]
        [DataType(DataType.Text)]
        [Display(Name = "Recovery Code")]
        public string RecoveryCode { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/ChangePassword.razor">
﻿@page "/Account/Manage/ChangePassword"

@using System.ComponentModel.DataAnnotations
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager
@inject ILogger<ChangePassword> Logger

<PageTitle>Change password</PageTitle>

<h3>Change password</h3>
<StatusMessage Message="@message" />
<div class="row">
    <div class="col-xl-6">
        <EditForm Model="Input" FormName="change-password" OnValidSubmit="OnValidSubmitAsync" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.OldPassword" id="Input.OldPassword" class="form-control" autocomplete="current-password" aria-required="true" placeholder="Enter the old password" />
                <label for="Input.OldPassword" class="form-label">Old password</label>
                <ValidationMessage For="() => Input.OldPassword" class="text-danger" />
            </div>
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.NewPassword" id="Input.NewPassword" class="form-control" autocomplete="new-password" aria-required="true" placeholder="Enter the new password" />
                <label for="Input.NewPassword" class="form-label">New password</label>
                <ValidationMessage For="() => Input.NewPassword" class="text-danger" />
            </div>
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.ConfirmPassword" id="Input.ConfirmPassword" class="form-control" autocomplete="new-password" aria-required="true" placeholder="Enter the new password" />
                <label for="Input.ConfirmPassword" class="form-label">Confirm password</label>
                <ValidationMessage For="() => Input.ConfirmPassword" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Update password</button>
        </EditForm>
    </div>
</div>

@code {
    private string? message;
    private ApplicationUser? user;
    private bool hasPassword;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        hasPassword = await UserManager.HasPasswordAsync(user);
        if (!hasPassword)
        {
            RedirectManager.RedirectTo("Account/Manage/SetPassword");
        }
    }

    private async Task OnValidSubmitAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var changePasswordResult = await UserManager.ChangePasswordAsync(user, Input.OldPassword, Input.NewPassword);
        if (!changePasswordResult.Succeeded)
        {
            message = $"Error: {string.Join(",", changePasswordResult.Errors.Select(error => error.Description))}";
            return;
        }

        await SignInManager.RefreshSignInAsync(user);
        Logger.LogInformation("User changed their password successfully.");

        RedirectManager.RedirectToCurrentPageWithStatus("Your password has been changed", HttpContext);
    }

    private sealed class InputModel
    {
        [Required]
        [DataType(DataType.Password)]
        [Display(Name = "Current password")]
        public string OldPassword { get; set; } = "";

        [Required]
        [StringLength(100, ErrorMessage = "The {0} must be at least {2} and at max {1} characters long.", MinimumLength = 6)]
        [DataType(DataType.Password)]
        [Display(Name = "New password")]
        public string NewPassword { get; set; } = "";

        [DataType(DataType.Password)]
        [Display(Name = "Confirm new password")]
        [Compare("NewPassword", ErrorMessage = "The new password and confirmation password do not match.")]
        public string ConfirmPassword { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/DeletePersonalData.razor">
﻿@page "/Account/Manage/DeletePersonalData"

@using System.ComponentModel.DataAnnotations
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager
@inject ILogger<DeletePersonalData> Logger

<PageTitle>Delete Personal Data</PageTitle>

<StatusMessage Message="@message" />

<h3>Delete Personal Data</h3>

<div class="alert alert-warning" role="alert">
    <p>
        <strong>Deleting this data will permanently remove your account, and this cannot be recovered.</strong>
    </p>
</div>

<div>
    <EditForm Model="Input" FormName="delete-user" OnValidSubmit="OnValidSubmitAsync" method="post">
        <DataAnnotationsValidator />
        <ValidationSummary class="text-danger" role="alert" />
        @if (requirePassword)
        {
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.Password" id="Input.Password" class="form-control" autocomplete="current-password" aria-required="true" placeholder="Please enter your password." />
                <label for="Input.Password" class="form-label">Password</label>
                <ValidationMessage For="() => Input.Password" class="text-danger" />
            </div>
        }
        <button class="w-100 btn btn-lg btn-danger" type="submit">Delete data and close my account</button>
    </EditForm>
</div>

@code {
    private string? message;
    private ApplicationUser? user;
    private bool requirePassword;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }
        requirePassword = await UserManager.HasPasswordAsync(user);
    }

    private async Task OnValidSubmitAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        if (requirePassword && !await UserManager.CheckPasswordAsync(user, Input.Password))
        {
            message = "Error: Incorrect password.";
            return;
        }

        var result = await UserManager.DeleteAsync(user);
        if (!result.Succeeded)
        {
            throw new InvalidOperationException("Unexpected error occurred deleting user.");
        }

        await SignInManager.SignOutAsync();

        var userId = await UserManager.GetUserIdAsync(user);
        Logger.LogInformation("User with ID '{UserId}' deleted themselves.", userId);

        RedirectManager.RedirectToCurrentPage();
    }

    private sealed class InputModel
    {
        [DataType(DataType.Password)]
        public string Password { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/Disable2fa.razor">
﻿@page "/Account/Manage/Disable2fa"

@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IdentityRedirectManager RedirectManager
@inject ILogger<Disable2fa> Logger

<PageTitle>Disable two-factor authentication (2FA)</PageTitle>

<StatusMessage />
<h3>Disable two-factor authentication (2FA)</h3>

<div class="alert alert-warning" role="alert">
    <p>
        <strong>This action only disables 2FA.</strong>
    </p>
    <p>
        Disabling 2FA does not change the keys used in authenticator apps. If you wish to change the key
        used in an authenticator app you should <a href="Account/Manage/ResetAuthenticator">reset your authenticator keys.</a>
    </p>
</div>

<div>
    <form @formname="disable-2fa" @onsubmit="OnSubmitAsync" method="post">
        <AntiforgeryToken />
        <button class="btn btn-danger" type="submit">Disable 2FA</button>
    </form>
</div>

@code {
    private ApplicationUser? user;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        if (HttpMethods.IsGet(HttpContext.Request.Method) && !await UserManager.GetTwoFactorEnabledAsync(user))
        {
            throw new InvalidOperationException("Cannot disable 2FA for user as it's not currently enabled.");
        }
    }

    private async Task OnSubmitAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var disable2faResult = await UserManager.SetTwoFactorEnabledAsync(user, false);
        if (!disable2faResult.Succeeded)
        {
            throw new InvalidOperationException("Unexpected error occurred disabling 2FA.");
        }

        var userId = await UserManager.GetUserIdAsync(user);
        Logger.LogInformation("User with ID '{UserId}' has disabled 2fa.", userId);
        RedirectManager.RedirectToWithStatus(
            "Account/Manage/TwoFactorAuthentication",
            "2fa has been disabled. You can reenable 2fa when you setup an authenticator app",
            HttpContext);
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/Email.razor">
﻿@page "/Account/Manage/Email"

@using System.ComponentModel.DataAnnotations
@using System.Text
@using System.Text.Encodings.Web
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IEmailSender<ApplicationUser> EmailSender
@inject NavigationManager NavigationManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Manage email</PageTitle>

<h3>Manage email</h3>

<StatusMessage Message="@message"/>
<div class="row">
    <div class="col-xl-6">
        <form @onsubmit="OnSendEmailVerificationAsync" @formname="send-verification" id="send-verification-form" method="post">
            <AntiforgeryToken />
        </form>
        <EditForm Model="Input" FormName="change-email" OnValidSubmit="OnValidSubmitAsync" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />
            @if (isEmailConfirmed)
            {
                <div class="form-floating mb-3 input-group">
                    <input type="text" value="@email" id="email" class="form-control" placeholder="Enter your email" disabled />
                    <div class="input-group-append">
                        <span class="h-100 input-group-text text-success font-weight-bold">✓</span>
                    </div>
                    <label for="email" class="form-label">Email</label>
                </div>
            }
            else
            {
                <div class="form-floating mb-3">
                    <input type="text" value="@email" id="email" class="form-control" placeholder="Enter your email" disabled />
                    <label for="email" class="form-label">Email</label>
                    <button type="submit" class="btn btn-link" form="send-verification-form">Send verification email</button>
                </div>
            }
            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.NewEmail" id="Input.NewEmail" class="form-control" autocomplete="email" aria-required="true" placeholder="Enter a new email" />
                <label for="Input.NewEmail" class="form-label">New email</label>
                <ValidationMessage For="() => Input.NewEmail" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Change email</button>
        </EditForm>
    </div>
</div>

@code {
    private string? message;
    private ApplicationUser? user;
    private string? email;
    private bool isEmailConfirmed;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm(FormName = "change-email")]
    private InputModel Input { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        email = await UserManager.GetEmailAsync(user);
        isEmailConfirmed = await UserManager.IsEmailConfirmedAsync(user);

        Input.NewEmail ??= email;
    }

    private async Task OnValidSubmitAsync()
    {
        if (Input.NewEmail is null || Input.NewEmail == email)
        {
            message = "Your email is unchanged.";
            return;
        }

        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var userId = await UserManager.GetUserIdAsync(user);
        var code = await UserManager.GenerateChangeEmailTokenAsync(user, Input.NewEmail);
        code = WebEncoders.Base64UrlEncode(Encoding.UTF8.GetBytes(code));
        var callbackUrl = NavigationManager.GetUriWithQueryParameters(
            NavigationManager.ToAbsoluteUri("Account/ConfirmEmailChange").AbsoluteUri,
            new Dictionary<string, object?> { ["userId"] = userId, ["email"] = Input.NewEmail, ["code"] = code });

        await EmailSender.SendConfirmationLinkAsync(user, Input.NewEmail, HtmlEncoder.Default.Encode(callbackUrl));

        message = "Confirmation link to change email sent. Please check your email.";
    }

    private async Task OnSendEmailVerificationAsync()
    {
        if (email is null)
        {
            return;
        }

        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var userId = await UserManager.GetUserIdAsync(user);
        var code = await UserManager.GenerateEmailConfirmationTokenAsync(user);
        code = WebEncoders.Base64UrlEncode(Encoding.UTF8.GetBytes(code));
        var callbackUrl = NavigationManager.GetUriWithQueryParameters(
            NavigationManager.ToAbsoluteUri("Account/ConfirmEmail").AbsoluteUri,
            new Dictionary<string, object?> { ["userId"] = userId, ["code"] = code });

        await EmailSender.SendConfirmationLinkAsync(user, email, HtmlEncoder.Default.Encode(callbackUrl));

        message = "Verification email sent. Please check your email.";
    }

    private sealed class InputModel
    {
        [Required]
        [EmailAddress]
        [Display(Name = "New email")]
        public string? NewEmail { get; set; }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/EnableAuthenticator.razor">
﻿@page "/Account/Manage/EnableAuthenticator"

@using System.ComponentModel.DataAnnotations
@using System.Globalization
@using System.Text
@using System.Text.Encodings.Web
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject UrlEncoder UrlEncoder
@inject IdentityRedirectManager RedirectManager
@inject ILogger<EnableAuthenticator> Logger

<PageTitle>Configure authenticator app</PageTitle>

@if (recoveryCodes is not null)
{
    <ShowRecoveryCodes RecoveryCodes="recoveryCodes.ToArray()" StatusMessage="@message" />
}
else
{
    <StatusMessage Message="@message" />
    <h3>Configure authenticator app</h3>
    <div>
        <p>To use an authenticator app go through the following steps:</p>
        <ol class="list">
            <li>
                <p>
                    Download a two-factor authenticator app like Microsoft Authenticator for
                    <a href="https://go.microsoft.com/fwlink/?Linkid=825072">Android</a> and
                    <a href="https://go.microsoft.com/fwlink/?Linkid=825073">iOS</a> or
                    Google Authenticator for
                    <a href="https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2&amp;hl=en">Android</a> and
                    <a href="https://itunes.apple.com/us/app/google-authenticator/id388497605?mt=8">iOS</a>.
                </p>
            </li>
            <li>
                <p>Scan the QR Code or enter this key <kbd>@sharedKey</kbd> into your two factor authenticator app. Spaces and casing do not matter.</p>
                <div class="alert alert-info">Learn how to <a href="https://go.microsoft.com/fwlink/?Linkid=852423">enable QR code generation</a>.</div>
                <div></div>
                <div data-url="@authenticatorUri"></div>
            </li>
            <li>
                <p>
                    Once you have scanned the QR code or input the key above, your two factor authentication app will provide you
                    with a unique code. Enter the code in the confirmation box below.
                </p>
                <div class="row">
                    <div class="col-xl-6">
                        <EditForm Model="Input" FormName="send-code" OnValidSubmit="OnValidSubmitAsync" method="post">
                            <DataAnnotationsValidator />
                            <div class="form-floating mb-3">
                                <InputText @bind-Value="Input.Code" id="Input.Code" class="form-control" autocomplete="off" placeholder="Enter the code" />
                                <label for="Input.Code" class="control-label form-label">Verification Code</label>
                                <ValidationMessage For="() => Input.Code" class="text-danger" />
                            </div>
                            <button type="submit" class="w-100 btn btn-lg btn-primary">Verify</button>
                            <ValidationSummary class="text-danger" role="alert" />
                        </EditForm>
                    </div>
                </div>
            </li>
        </ol>
    </div>
}

@code {
    private const string AuthenticatorUriFormat = "otpauth://totp/{0}:{1}?secret={2}&issuer={0}&digits=6";

    private string? message;
    private ApplicationUser? user;
    private string? sharedKey;
    private string? authenticatorUri;
    private IEnumerable<string>? recoveryCodes;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        await LoadSharedKeyAndQrCodeUriAsync(user);
    }

    private async Task OnValidSubmitAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        // Strip spaces and hyphens
        var verificationCode = Input.Code.Replace(" ", string.Empty).Replace("-", string.Empty);

        var is2faTokenValid = await UserManager.VerifyTwoFactorTokenAsync(
            user, UserManager.Options.Tokens.AuthenticatorTokenProvider, verificationCode);

        if (!is2faTokenValid)
        {
            message = "Error: Verification code is invalid.";
            return;
        }

        await UserManager.SetTwoFactorEnabledAsync(user, true);
        var userId = await UserManager.GetUserIdAsync(user);
        Logger.LogInformation("User with ID '{UserId}' has enabled 2FA with an authenticator app.", userId);

        message = "Your authenticator app has been verified.";

        if (await UserManager.CountRecoveryCodesAsync(user) == 0)
        {
            recoveryCodes = await UserManager.GenerateNewTwoFactorRecoveryCodesAsync(user, 10);
        }
        else
        {
            RedirectManager.RedirectToWithStatus("Account/Manage/TwoFactorAuthentication", message, HttpContext);
        }
    }

    private async ValueTask LoadSharedKeyAndQrCodeUriAsync(ApplicationUser user)
    {
        // Load the authenticator key & QR code URI to display on the form
        var unformattedKey = await UserManager.GetAuthenticatorKeyAsync(user);
        if (string.IsNullOrEmpty(unformattedKey))
        {
            await UserManager.ResetAuthenticatorKeyAsync(user);
            unformattedKey = await UserManager.GetAuthenticatorKeyAsync(user);
        }

        sharedKey = FormatKey(unformattedKey!);

        var email = await UserManager.GetEmailAsync(user);
        authenticatorUri = GenerateQrCodeUri(email!, unformattedKey!);
    }

    private string FormatKey(string unformattedKey)
    {
        var result = new StringBuilder();
        int currentPosition = 0;
        while (currentPosition + 4 < unformattedKey.Length)
        {
            result.Append(unformattedKey.AsSpan(currentPosition, 4)).Append(' ');
            currentPosition += 4;
        }
        if (currentPosition < unformattedKey.Length)
        {
            result.Append(unformattedKey.AsSpan(currentPosition));
        }

        return result.ToString().ToLowerInvariant();
    }

    private string GenerateQrCodeUri(string email, string unformattedKey)
    {
        return string.Format(
            CultureInfo.InvariantCulture,
            AuthenticatorUriFormat,
            UrlEncoder.Encode("Microsoft.AspNetCore.Identity.UI"),
            UrlEncoder.Encode(email),
            unformattedKey);
    }

    private sealed class InputModel
    {
        [Required]
        [StringLength(7, ErrorMessage = "The {0} must be at least {2} and at max {1} characters long.", MinimumLength = 6)]
        [DataType(DataType.Text)]
        [Display(Name = "Verification Code")]
        public string Code { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/ExternalLogins.razor">
﻿@page "/Account/Manage/ExternalLogins"

@using Microsoft.AspNetCore.Authentication
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IUserStore<ApplicationUser> UserStore
@inject IdentityRedirectManager RedirectManager

<PageTitle>Manage your external logins</PageTitle>

<StatusMessage />
@if (currentLogins?.Count > 0)
{
    <h3>Registered Logins</h3>
    <table class="table">
        <tbody>
            @foreach (var login in currentLogins)
            {
                <tr>
                    <td>@login.ProviderDisplayName</td>
                    <td>
                        @if (showRemoveButton)
                        {
                            <form @formname="@($"remove-login-{login.LoginProvider}")" @onsubmit="OnSubmitAsync" method="post">
                                <AntiforgeryToken />
                                <div>
                                    <input type="hidden" name="@nameof(LoginProvider)" value="@login.LoginProvider" />
                                    <input type="hidden" name="@nameof(ProviderKey)" value="@login.ProviderKey" />
                                    <button type="submit" class="btn btn-primary" title="Remove this @login.ProviderDisplayName login from your account">Remove</button>
                                </div>
                            </form>
                        }
                        else
                        {
                            @: &nbsp;
                        }
                    </td>
                </tr>
            }
        </tbody>
    </table>
}
@if (otherLogins?.Count > 0)
{
    <h4>Add another service to log in.</h4>
    <hr />
    <form class="form-horizontal" action="Account/Manage/LinkExternalLogin" method="post">
        <AntiforgeryToken />
        <div>
            <p>
                @foreach (var provider in otherLogins)
                {
                    <button type="submit" class="btn btn-primary" name="Provider" value="@provider.Name" title="Log in using your @provider.DisplayName account">
                        @provider.DisplayName
                    </button>
                }
            </p>
        </div>
    </form>
}

@code {
    public const string LinkLoginCallbackAction = "LinkLoginCallback";

    private ApplicationUser? user;
    private IList<UserLoginInfo>? currentLogins;
    private IList<AuthenticationScheme>? otherLogins;
    private bool showRemoveButton;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private string? LoginProvider { get; set; }

    [SupplyParameterFromForm]
    private string? ProviderKey { get; set; }

    [SupplyParameterFromQuery]
    private string? Action { get; set; }

    protected override async Task OnInitializedAsync()
    {
        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        currentLogins = await UserManager.GetLoginsAsync(user);
        otherLogins = (await SignInManager.GetExternalAuthenticationSchemesAsync())
            .Where(auth => currentLogins.All(ul => auth.Name != ul.LoginProvider))
            .ToList();

        string? passwordHash = null;
        if (UserStore is IUserPasswordStore<ApplicationUser> userPasswordStore)
        {
            passwordHash = await userPasswordStore.GetPasswordHashAsync(user, HttpContext.RequestAborted);
        }

        showRemoveButton = passwordHash is not null || currentLogins.Count > 1;

        if (HttpMethods.IsGet(HttpContext.Request.Method) && Action == LinkLoginCallbackAction)
        {
            await OnGetLinkLoginCallbackAsync();
        }
    }

    private async Task OnSubmitAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var result = await UserManager.RemoveLoginAsync(user, LoginProvider!, ProviderKey!);
        if (!result.Succeeded)
        {
            RedirectManager.RedirectToCurrentPageWithStatus("Error: The external login was not removed.", HttpContext);
        }
        else
        {
            await SignInManager.RefreshSignInAsync(user);
            RedirectManager.RedirectToCurrentPageWithStatus("The external login was removed.", HttpContext);
        }
    }

    private async Task OnGetLinkLoginCallbackAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var userId = await UserManager.GetUserIdAsync(user);
        var info = await SignInManager.GetExternalLoginInfoAsync(userId);
        if (info is null)
        {
            RedirectManager.RedirectToCurrentPageWithStatus("Error: Could not load external login info.", HttpContext);
            return;
        }

        var result = await UserManager.AddLoginAsync(user, info);
        if (result.Succeeded)
        {
            // Clear the existing external cookie to ensure a clean login process
            await HttpContext.SignOutAsync(IdentityConstants.ExternalScheme);

            RedirectManager.RedirectToCurrentPageWithStatus("The external login was added.", HttpContext);
        }
        else
        {
            RedirectManager.RedirectToCurrentPageWithStatus("Error: The external login was not added. External logins can only be associated with one account.", HttpContext);
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/GenerateRecoveryCodes.razor">
﻿@page "/Account/Manage/GenerateRecoveryCodes"

@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IdentityRedirectManager RedirectManager
@inject ILogger<GenerateRecoveryCodes> Logger

<PageTitle>Generate two-factor authentication (2FA) recovery codes</PageTitle>

@if (recoveryCodes is not null)
{
    <ShowRecoveryCodes RecoveryCodes="recoveryCodes.ToArray()" StatusMessage="@message" />
}
else
{
    <h3>Generate two-factor authentication (2FA) recovery codes</h3>
    <div class="alert alert-warning" role="alert">
        <p>
            <span class="glyphicon glyphicon-warning-sign"></span>
            <strong>Put these codes in a safe place.</strong>
        </p>
        <p>
            If you lose your device and don't have the recovery codes you will lose access to your account.
        </p>
        <p>
            Generating new recovery codes does not change the keys used in authenticator apps. If you wish to change the key
            used in an authenticator app you should <a href="Account/Manage/ResetAuthenticator">reset your authenticator keys.</a>
        </p>
    </div>
    <div>
        <form @formname="generate-recovery-codes" @onsubmit="OnSubmitAsync" method="post">
            <AntiforgeryToken />
            <button class="btn btn-danger" type="submit">Generate Recovery Codes</button>
        </form>
    </div>
}

@code {
    private string? message;
    private ApplicationUser? user;
    private IEnumerable<string>? recoveryCodes;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var isTwoFactorEnabled = await UserManager.GetTwoFactorEnabledAsync(user);
        if (!isTwoFactorEnabled)
        {
            throw new InvalidOperationException("Cannot generate recovery codes for user because they do not have 2FA enabled.");
        }
    }

    private async Task OnSubmitAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var userId = await UserManager.GetUserIdAsync(user);
        recoveryCodes = await UserManager.GenerateNewTwoFactorRecoveryCodesAsync(user, 10);
        message = "You have generated new recovery codes.";

        Logger.LogInformation("User with ID '{UserId}' has generated new 2FA recovery codes.", userId);
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/Index.razor">
﻿@page "/Account/Manage"

@using System.ComponentModel.DataAnnotations
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Profile</PageTitle>

<h3>Profile</h3>
<StatusMessage />

<div class="row">
    <div class="col-xl-6">
        <EditForm Model="Input" FormName="profile" OnValidSubmit="OnValidSubmitAsync" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />
            <div class="form-floating mb-3">
                <input type="text" value="@username" id="username" class="form-control" placeholder="Choose your username." disabled />
                <label for="username" class="form-label">Username</label>
            </div>
            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.PhoneNumber" id="Input.PhoneNumber" class="form-control" placeholder="Enter your phone number" />
                <label for="Input.PhoneNumber" class="form-label">Phone number</label>
                <ValidationMessage For="() => Input.PhoneNumber" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Save</button>
        </EditForm>
    </div>
</div>

@code {
    private ApplicationUser? user;
    private string? username;
    private string? phoneNumber;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        username = await UserManager.GetUserNameAsync(user);
        phoneNumber = await UserManager.GetPhoneNumberAsync(user);

        Input.PhoneNumber ??= phoneNumber;
    }

    private async Task OnValidSubmitAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        if (Input.PhoneNumber != phoneNumber)
        {
            var setPhoneResult = await UserManager.SetPhoneNumberAsync(user, Input.PhoneNumber);
            if (!setPhoneResult.Succeeded)
            {
                RedirectManager.RedirectToCurrentPageWithStatus("Error: Failed to set phone number.", HttpContext);
                return;
            }
        }

        await SignInManager.RefreshSignInAsync(user);
        RedirectManager.RedirectToCurrentPageWithStatus("Your profile has been updated", HttpContext);
    }

    private sealed class InputModel
    {
        [Phone]
        [Display(Name = "Phone number")]
        public string? PhoneNumber { get; set; }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/Passkeys.razor">
﻿@page "/Account/Manage/Passkeys"

@using HickoryTodos.Data
@using Microsoft.AspNetCore.Identity
@using System.ComponentModel.DataAnnotations
@using System.Buffers.Text

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Manage your passkeys</PageTitle>

<h3>Manage your passkeys</h3>

<StatusMessage />

@if (currentPasskeys is { Count: > 0 })
{
    <table class="table">
        <tbody>
            @foreach (var passkey in currentPasskeys)
            {
                <tr>
                    <td>@(passkey.Name ?? "Unnamed passkey")</td>
                    <td>
                        @{
                            var credentialId = Base64Url.EncodeToString(passkey.CredentialId);
                        }
                        <form @formname="@($"update-passkey-{credentialId}")" @onsubmit="UpdatePasskey" method="post">
                            <AntiforgeryToken />
                            <div>
                                <input type="hidden" name="CredentialId" value="@credentialId" />
                                <button type="submit" name="Action" value="rename" class="btn btn-primary" title="Rename this passkey">Rename</button>
                                <button type="submit" name="Action" value="delete" class="btn btn-danger" title="Remove this passkey from your account">Delete</button>
                            </div>
                        </form>
                    </td>
                </tr>
            }
        </tbody>
    </table>
}
else
{
    <p>No passkeys are registered.</p>
}

<form @formname="add-passkey" @onsubmit="AddPasskey" method="post">
    <AntiforgeryToken />
    @if (currentPasskeys is { Count: >= MaxPasskeyCount })
    {
        <p class="text-danger">You have reached the maximum number of allowed passkeys. Please delete one before adding a new one.</p>
    }
    else
    {
        <PasskeySubmit Operation="PasskeyOperation.Create" Name="Input" class="btn btn-primary">Add a new passkey</PasskeySubmit>
    }

</form>

@code {
    private const int MaxPasskeyCount = 100;

    private ApplicationUser? user;
    private IList<UserPasskeyInfo>? currentPasskeys;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private string? Action { get; set; }

    [SupplyParameterFromForm]
    private string? CredentialId { get; set; }

    [SupplyParameterFromForm(FormName = "add-passkey")]
    private PasskeyInputModel Input { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }
        currentPasskeys = await UserManager.GetPasskeysAsync(user);
    }

    private async Task AddPasskey()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        if (!string.IsNullOrEmpty(Input.Error))
        {
            RedirectManager.RedirectToCurrentPageWithStatus($"Error: {Input.Error}", HttpContext);
            return;
        }

        if (string.IsNullOrEmpty(Input.CredentialJson))
        {
            RedirectManager.RedirectToCurrentPageWithStatus("Error: The browser did not provide a passkey.", HttpContext);
            return;
        }

        if (currentPasskeys!.Count >= MaxPasskeyCount)
        {
            RedirectManager.RedirectToCurrentPageWithStatus($"Error: You have reached the maximum number of allowed passkeys.", HttpContext);
            return;
        }

        var attestationResult = await SignInManager.PerformPasskeyAttestationAsync(Input.CredentialJson);
        if (!attestationResult.Succeeded)
        {
            RedirectManager.RedirectToCurrentPageWithStatus($"Error: Could not add the passkey: {attestationResult.Failure.Message}", HttpContext);
            return;
        }

        var addPasskeyResult = await UserManager.AddOrUpdatePasskeyAsync(user, attestationResult.Passkey);
        if (!addPasskeyResult.Succeeded)
        {
            RedirectManager.RedirectToCurrentPageWithStatus("Error: The passkey could not be added to your account.", HttpContext);
            return;
        }

        // Immediately prompt the user to enter a name for the credential
        var credentialIdBase64Url = Base64Url.EncodeToString(attestationResult.Passkey.CredentialId);
        RedirectManager.RedirectTo($"Account/Manage/RenamePasskey/{credentialIdBase64Url}");
    }

    private async Task UpdatePasskey()
    {
        switch (Action)
        {
            case "rename":
                RedirectManager.RedirectTo($"Account/Manage/RenamePasskey/{CredentialId}");
                break;
            case "delete":
                await DeletePasskey();
                break;
            default:
                RedirectManager.RedirectToCurrentPageWithStatus($"Error: Unknown action '{Action}'.", HttpContext);
                break;
        }
    }

    private async Task DeletePasskey()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        byte[] credentialId;
        try
        {
            credentialId = Base64Url.DecodeFromChars(CredentialId);
        }
        catch (FormatException)
        {
            RedirectManager.RedirectToCurrentPageWithStatus("Error: The specified passkey ID had an invalid format.", HttpContext);
            return;
        }

        var result = await UserManager.RemovePasskeyAsync(user, credentialId);
        if (!result.Succeeded)
        {
            RedirectManager.RedirectToCurrentPageWithStatus("Error: The passkey could not be deleted.", HttpContext);
            return;
        }

        RedirectManager.RedirectToCurrentPageWithStatus("Passkey deleted successfully.", HttpContext);
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/PersonalData.razor">
﻿@page "/Account/Manage/PersonalData"

@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Personal Data</PageTitle>

<StatusMessage />
<h3>Personal Data</h3>

<div class="row">
    <div class="col-md-6">
        <p>Your account contains personal data that you have given us. This page allows you to download or delete that data.</p>
        <p>
            <strong>Deleting this data will permanently remove your account, and this cannot be recovered.</strong>
        </p>
        <form action="Account/Manage/DownloadPersonalData" method="post">
            <AntiforgeryToken />
            <button class="btn btn-primary" type="submit">Download</button>
        </form>
        <p>
            <a href="Account/Manage/DeletePersonalData" class="btn btn-danger">Delete</a>
        </p>
    </div>
</div>

@code {
    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        var user  = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/RenamePasskey.razor">
﻿@page "/Account/Manage/RenamePasskey/{Id}"

@using HickoryTodos.Data
@using System.ComponentModel.DataAnnotations
@using Microsoft.AspNetCore.Identity
@using System.Buffers.Text

@inject UserManager<ApplicationUser> UserManager
@inject IdentityRedirectManager RedirectManager

<EditForm Model="Input" OnValidSubmit="Rename" FormName="rename-passkey" method="post">
    <DataAnnotationsValidator />
    @if (passkey?.Name is { } name)
    {
        <h4>Enter a new name for your "@name" passkey</h4>
    }
    else
    {
        <h4>Enter a name for your passkey</h4>
    }
    <hr />
    <ValidationSummary class="text-danger" role="alert" />
    <div class="form-floating mb-3">
        <InputText @bind-Value="Input.Name" id="Input.Name" class="form-control" aria-required="true" placeholder="My passkey" />
        <label for="Input.Name" class="form-label">Passkey name</label>
        <ValidationMessage For="() => Input.Name" class="text-danger" />
    </div>
    <div>
        <button type="submit" class="w-100 btn btn-lg btn-primary">Continue</button>
    </div>
</EditForm>

@code {
    private ApplicationUser? user;
    private UserPasskeyInfo? passkey;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [Parameter]
    public string? Id { get; set; }

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        user = (await UserManager.GetUserAsync(HttpContext.User))!;
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        byte[] credentialId;
        try
        {
            credentialId = Base64Url.DecodeFromChars(Id);
        }
        catch (FormatException)
        {
            RedirectManager.RedirectToWithStatus("Account/Manage/Passkeys", "Error: The specified passkey ID had an invalid format.", HttpContext);
            return;
        }

        passkey = await UserManager.GetPasskeyAsync(user, credentialId);
        if (passkey is null)
        {
            RedirectManager.RedirectToWithStatus("Account/Manage/Passkeys", "Error: The specified passkey could not be found.", HttpContext);
            return;
        }
    }

    private async Task Rename()
    {
        passkey!.Name = Input.Name;
        var result = await UserManager.AddOrUpdatePasskeyAsync(user!, passkey);
        if (!result.Succeeded)
        {
            RedirectManager.RedirectToWithStatus("Account/Manage/Passkeys", "Error: The passkey could not be updated.", HttpContext);
            return;
        }

        RedirectManager.RedirectToWithStatus("Account/Manage/Passkeys", "Passkey updated successfully.", HttpContext);
    }

    private sealed class InputModel
    {
        [Required]
        [StringLength(200, ErrorMessage = "Passkey names must be no longer than {1} characters.")]
        public string Name { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/ResetAuthenticator.razor">
﻿@page "/Account/Manage/ResetAuthenticator"

@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager
@inject ILogger<ResetAuthenticator> Logger

<PageTitle>Reset authenticator key</PageTitle>

<StatusMessage />
<h3>Reset authenticator key</h3>
<div class="alert alert-warning" role="alert">
    <p>
        <span class="glyphicon glyphicon-warning-sign"></span>
        <strong>If you reset your authenticator key your authenticator app will not work until you reconfigure it.</strong>
    </p>
    <p>
        This process disables 2FA until you verify your authenticator app.
        If you do not complete your authenticator app configuration you may lose access to your account.
    </p>
</div>
<div>
    <form @formname="reset-authenticator" @onsubmit="OnSubmitAsync" method="post">
        <AntiforgeryToken />
        <button class="btn btn-danger" type="submit">Reset authenticator key</button>
    </form>
</div>

@code {
    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    private async Task OnSubmitAsync()
    {
        var user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        await UserManager.SetTwoFactorEnabledAsync(user, false);
        await UserManager.ResetAuthenticatorKeyAsync(user);
        var userId = await UserManager.GetUserIdAsync(user);
        Logger.LogInformation("User with ID '{UserId}' has reset their authentication app key.", userId);

        await SignInManager.RefreshSignInAsync(user);

        RedirectManager.RedirectToWithStatus(
            "Account/Manage/EnableAuthenticator",
            "Your authenticator app key has been reset, you will need to configure your authenticator app using the new key.",
            HttpContext);
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/SetPassword.razor">
﻿@page "/Account/Manage/SetPassword"

@using System.ComponentModel.DataAnnotations
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Set password</PageTitle>

<h3>Set your password</h3>
<StatusMessage Message="@message" />
<p class="text-info">
    You do not have a local username/password for this site. Add a local
    account so you can log in without an external login.
</p>
<div class="row">
    <div class="col-xl-6">
        <EditForm Model="Input" FormName="set-password" OnValidSubmit="OnValidSubmitAsync" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.NewPassword" id="Input.NewPassword" class="form-control" autocomplete="new-password" placeholder="Enter the new password" />
                <label for="Input.NewPassword" class="form-label">New password</label>
                <ValidationMessage For="() => Input.NewPassword" class="text-danger" />
            </div>
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.ConfirmPassword" id="Input.ConfirmPassword" class="form-control" autocomplete="new-password" placeholder="Enter the new password" />
                <label for="Input.ConfirmPassword" class="form-label">Confirm password</label>
                <ValidationMessage For="() => Input.ConfirmPassword" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Set password</button>
        </EditForm>
     </div>
</div>

@code {
    private string? message;
    private ApplicationUser? user;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        Input ??= new();

        user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var hasPassword = await UserManager.HasPasswordAsync(user);
        if (hasPassword)
        {
            RedirectManager.RedirectTo("Account/Manage/ChangePassword");
        }
    }

    private async Task OnValidSubmitAsync()
    {
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        var addPasswordResult = await UserManager.AddPasswordAsync(user, Input.NewPassword!);
        if (!addPasswordResult.Succeeded)
        {
            message = $"Error: {string.Join(",", addPasswordResult.Errors.Select(error => error.Description))}";
            return;
        }

        await SignInManager.RefreshSignInAsync(user);
        RedirectManager.RedirectToCurrentPageWithStatus("Your password has been set.", HttpContext);
    }

    private sealed class InputModel
    {
        [Required]
        [StringLength(100, ErrorMessage = "The {0} must be at least {2} and at max {1} characters long.", MinimumLength = 6)]
        [DataType(DataType.Password)]
        [Display(Name = "New password")]
        public string? NewPassword { get; set; }

        [DataType(DataType.Password)]
        [Display(Name = "Confirm new password")]
        [Compare("NewPassword", ErrorMessage = "The new password and confirmation password do not match.")]
        public string? ConfirmPassword { get; set; }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/TwoFactorAuthentication.razor">
﻿@page "/Account/Manage/TwoFactorAuthentication"

@using Microsoft.AspNetCore.Http.Features
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Two-factor authentication (2FA)</PageTitle>

<StatusMessage />
<h3>Two-factor authentication (2FA)</h3>
@if (canTrack)
{
    if (is2faEnabled)
    {
        if (recoveryCodesLeft == 0)
        {
            <div class="alert alert-danger">
                <strong>You have no recovery codes left.</strong>
                <p>You must <a href="Account/Manage/GenerateRecoveryCodes">generate a new set of recovery codes</a> before you can log in with a recovery code.</p>
            </div>
        }
        else if (recoveryCodesLeft == 1)
        {
            <div class="alert alert-danger">
                <strong>You have 1 recovery code left.</strong>
                <p>You can <a href="Account/Manage/GenerateRecoveryCodes">generate a new set of recovery codes</a>.</p>
            </div>
        }
        else if (recoveryCodesLeft <= 3)
        {
            <div class="alert alert-warning">
                <strong>You have @recoveryCodesLeft recovery codes left.</strong>
                <p>You should <a href="Account/Manage/GenerateRecoveryCodes">generate a new set of recovery codes</a>.</p>
            </div>
        }

        if (isMachineRemembered)
        {
            <form style="display: inline-block" @formname="forget-browser" @onsubmit="OnSubmitForgetBrowserAsync" method="post">
                <AntiforgeryToken />
                <button type="submit" class="btn btn-primary">Forget this browser</button>
            </form>
        }

        <a href="Account/Manage/Disable2fa" class="btn btn-primary">Disable 2FA</a>
        <a href="Account/Manage/GenerateRecoveryCodes" class="btn btn-primary">Reset recovery codes</a>
    }

    <h4>Authenticator app</h4>
    @if (!hasAuthenticator)
    {
        <a href="Account/Manage/EnableAuthenticator" class="btn btn-primary">Add authenticator app</a>
    }
    else
    {
        <a href="Account/Manage/EnableAuthenticator" class="btn btn-primary">Set up authenticator app</a>
        <a href="Account/Manage/ResetAuthenticator" class="btn btn-primary">Reset authenticator app</a>
    }
}
else
{
    <div class="alert alert-danger">
        <strong>Privacy and cookie policy have not been accepted.</strong>
        <p>You must accept the policy before you can enable two factor authentication.</p>
    </div>
}

@code {
    private bool canTrack;
    private bool hasAuthenticator;
    private int recoveryCodesLeft;
    private bool is2faEnabled;
    private bool isMachineRemembered;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    protected override async Task OnInitializedAsync()
    {
        var user = await UserManager.GetUserAsync(HttpContext.User);
        if (user is null)
        {
            RedirectManager.RedirectToInvalidUser(UserManager, HttpContext);
            return;
        }

        canTrack = HttpContext.Features.Get<ITrackingConsentFeature>()?.CanTrack ?? true;
        hasAuthenticator = await UserManager.GetAuthenticatorKeyAsync(user) is not null;
        is2faEnabled = await UserManager.GetTwoFactorEnabledAsync(user);
        isMachineRemembered = await SignInManager.IsTwoFactorClientRememberedAsync(user);
        recoveryCodesLeft = await UserManager.CountRecoveryCodesAsync(user);
    }

    private async Task OnSubmitForgetBrowserAsync()
    {
        await SignInManager.ForgetTwoFactorClientAsync();

        RedirectManager.RedirectToCurrentPageWithStatus(
            "The current browser has been forgotten. When you login again from this browser you will be prompted for your 2fa code.",
            HttpContext);
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Manage/_Imports.razor">
﻿@layout ManageLayout
@attribute [Microsoft.AspNetCore.Authorization.Authorize]
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/Register.razor">
﻿@page "/Account/Register"

@using System.ComponentModel.DataAnnotations
@using System.Text
@using System.Text.Encodings.Web
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IUserStore<ApplicationUser> UserStore
@inject SignInManager<ApplicationUser> SignInManager
@inject IEmailSender<ApplicationUser> EmailSender
@inject ILogger<Register> Logger
@inject NavigationManager NavigationManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Register</PageTitle>

<h1>Register</h1>

<div class="row">
    <div class="col-lg-6">
        <StatusMessage Message="@Message" />
        <EditForm Model="Input" method="post" OnValidSubmit="RegisterUser" FormName="register">
            <DataAnnotationsValidator />
            <h2>Create a new account.</h2>
            <hr />
            <ValidationSummary class="text-danger" role="alert" />
            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.Email" id="Input.Email" class="form-control" autocomplete="username" aria-required="true" placeholder="name@example.com" />
                <label for="Input.Email">Email</label>
                <ValidationMessage For="() => Input.Email" class="text-danger" />
            </div>
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.Password" id="Input.Password" class="form-control" autocomplete="new-password" aria-required="true" placeholder="password" />
                <label for="Input.Password">Password</label>
                <ValidationMessage For="() => Input.Password" class="text-danger" />
            </div>
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.ConfirmPassword" id="Input.ConfirmPassword" class="form-control" autocomplete="new-password" aria-required="true" placeholder="password" />
                <label for="Input.ConfirmPassword">Confirm Password</label>
                <ValidationMessage For="() => Input.ConfirmPassword" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Register</button>
        </EditForm>
    </div>
    <div class="col-lg-4 col-lg-offset-2">
        <section>
            <h3>Use another service to register.</h3>
            <hr />
            <ExternalLoginPicker />
        </section>
    </div>
</div>

@code {
    private IEnumerable<IdentityError>? identityErrors;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? ReturnUrl { get; set; }

    private string? Message => identityErrors is null ? null : $"Error: {string.Join(", ", identityErrors.Select(error => error.Description))}";

    protected override void OnInitialized()
    {
        Input ??= new();
    }

    public async Task RegisterUser(EditContext editContext)
    {
        var user = CreateUser();

        await UserStore.SetUserNameAsync(user, Input.Email, CancellationToken.None);
        var emailStore = GetEmailStore();
        await emailStore.SetEmailAsync(user, Input.Email, CancellationToken.None);
        var result = await UserManager.CreateAsync(user, Input.Password);

        if (!result.Succeeded)
        {
            identityErrors = result.Errors;
            return;
        }

        Logger.LogInformation("User created a new account with password.");

        var userId = await UserManager.GetUserIdAsync(user);
        var code = await UserManager.GenerateEmailConfirmationTokenAsync(user);
        code = WebEncoders.Base64UrlEncode(Encoding.UTF8.GetBytes(code));
        var callbackUrl = NavigationManager.GetUriWithQueryParameters(
            NavigationManager.ToAbsoluteUri("Account/ConfirmEmail").AbsoluteUri,
            new Dictionary<string, object?> { ["userId"] = userId, ["code"] = code, ["returnUrl"] = ReturnUrl });

        await EmailSender.SendConfirmationLinkAsync(user, Input.Email, HtmlEncoder.Default.Encode(callbackUrl));

        if (UserManager.Options.SignIn.RequireConfirmedAccount)
        {
            RedirectManager.RedirectTo(
                "Account/RegisterConfirmation",
                new() { ["email"] = Input.Email, ["returnUrl"] = ReturnUrl });
        }
        else
        {
            await SignInManager.SignInAsync(user, isPersistent: false);
            RedirectManager.RedirectTo(ReturnUrl);
        }
    }

    private ApplicationUser CreateUser()
    {
        try
        {
            return Activator.CreateInstance<ApplicationUser>();
        }
        catch
        {
            throw new InvalidOperationException($"Can't create an instance of '{nameof(ApplicationUser)}'. " +
                $"Ensure that '{nameof(ApplicationUser)}' is not an abstract class and has a parameterless constructor.");
        }
    }

    private IUserEmailStore<ApplicationUser> GetEmailStore()
    {
        if (!UserManager.SupportsUserEmail)
        {
            throw new NotSupportedException("The default UI requires a user store with email support.");
        }
        return (IUserEmailStore<ApplicationUser>)UserStore;
    }

    private sealed class InputModel
    {
        [Required]
        [EmailAddress]
        [Display(Name = "Email")]
        public string Email { get; set; } = "";

        [Required]
        [StringLength(100, ErrorMessage = "The {0} must be at least {2} and at max {1} characters long.", MinimumLength = 6)]
        [DataType(DataType.Password)]
        [Display(Name = "Password")]
        public string Password { get; set; } = "";

        [DataType(DataType.Password)]
        [Display(Name = "Confirm password")]
        [Compare("Password", ErrorMessage = "The password and confirmation password do not match.")]
        public string ConfirmPassword { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/RegisterConfirmation.razor">
﻿@page "/Account/RegisterConfirmation"

@using System.Text
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IEmailSender<ApplicationUser> EmailSender
@inject NavigationManager NavigationManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Register confirmation</PageTitle>

<h1>Register confirmation</h1>

<StatusMessage Message="@statusMessage" />

@if (emailConfirmationLink is not null)
{
    <p>
        This app does not currently have a real email sender registered, see <a href="https://aka.ms/aspaccountconf">these docs</a> for how to configure a real email sender.
        Normally this would be emailed: <a href="@emailConfirmationLink">Click here to confirm your account</a>
    </p>
}
else
{
    <p role="alert">Please check your email to confirm your account.</p>
}

@code {
    private string? emailConfirmationLink;
    private string? statusMessage;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? Email { get; set; }

    [SupplyParameterFromQuery]
    private string? ReturnUrl { get; set; }

    protected override async Task OnInitializedAsync()
    {
        if (Email is null)
        {
            RedirectManager.RedirectTo("");
            return;
        }

        var user = await UserManager.FindByEmailAsync(Email);
        if (user is null)
        {
            HttpContext.Response.StatusCode = StatusCodes.Status404NotFound;
            statusMessage = "Error finding user for unspecified email";
        }
        else if (EmailSender is IdentityNoOpEmailSender)
        {
            // Once you add a real email sender, you should remove this code that lets you confirm the account
            var userId = await UserManager.GetUserIdAsync(user);
            var code = await UserManager.GenerateEmailConfirmationTokenAsync(user);
            code = WebEncoders.Base64UrlEncode(Encoding.UTF8.GetBytes(code));
            emailConfirmationLink = NavigationManager.GetUriWithQueryParameters(
                NavigationManager.ToAbsoluteUri("Account/ConfirmEmail").AbsoluteUri,
                new Dictionary<string, object?> { ["userId"] = userId, ["code"] = code, ["returnUrl"] = ReturnUrl });
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/ResendEmailConfirmation.razor">
﻿@page "/Account/ResendEmailConfirmation"

@using System.ComponentModel.DataAnnotations
@using System.Text
@using System.Text.Encodings.Web
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject UserManager<ApplicationUser> UserManager
@inject IEmailSender<ApplicationUser> EmailSender
@inject NavigationManager NavigationManager
@inject IdentityRedirectManager RedirectManager

<PageTitle>Resend email confirmation</PageTitle>

<h1>Resend email confirmation</h1>
<h2>Enter your email.</h2>
<hr />
<StatusMessage Message="@message" />
<div class="row">
    <div class="col-md-4">
        <EditForm Model="Input" FormName="resend-email-confirmation" OnValidSubmit="OnValidSubmitAsync" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />
            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.Email" id="Input.Email" class="form-control" aria-required="true" placeholder="name@example.com" />
                <label for="Input.Email" class="form-label">Email</label>
                <ValidationMessage For="() => Input.Email" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Resend</button>
        </EditForm>
    </div>
</div>

@code {
    private string? message;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    protected override void OnInitialized()
    {
        Input ??= new();
    }

    private async Task OnValidSubmitAsync()
    {
        var user = await UserManager.FindByEmailAsync(Input.Email!);
        if (user is null)
        {
            message = "Verification email sent. Please check your email.";
            return;
        }

        var userId = await UserManager.GetUserIdAsync(user);
        var code = await UserManager.GenerateEmailConfirmationTokenAsync(user);
        code = WebEncoders.Base64UrlEncode(Encoding.UTF8.GetBytes(code));
        var callbackUrl = NavigationManager.GetUriWithQueryParameters(
            NavigationManager.ToAbsoluteUri("Account/ConfirmEmail").AbsoluteUri,
            new Dictionary<string, object?> { ["userId"] = userId, ["code"] = code });
        await EmailSender.SendConfirmationLinkAsync(user, Input.Email, HtmlEncoder.Default.Encode(callbackUrl));

        message = "Verification email sent. Please check your email.";
    }

    private sealed class InputModel
    {
        [Required]
        [EmailAddress]
        public string Email { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/ResetPassword.razor">
﻿@page "/Account/ResetPassword"

@using System.ComponentModel.DataAnnotations
@using System.Text
@using Microsoft.AspNetCore.Identity
@using Microsoft.AspNetCore.WebUtilities
@using HickoryTodos.Data

@inject IdentityRedirectManager RedirectManager
@inject UserManager<ApplicationUser> UserManager

<PageTitle>Reset password</PageTitle>

<h1>Reset password</h1>
<h2>Reset your password.</h2>
<hr />
<div class="row">
    <div class="col-md-4">
        <StatusMessage Message="@Message" />
        <EditForm Model="Input" FormName="reset-password" OnValidSubmit="OnValidSubmitAsync" method="post">
            <DataAnnotationsValidator />
            <ValidationSummary class="text-danger" role="alert" />

            <input type="hidden" name="Input.Code" value="@Input.Code" />
            <div class="form-floating mb-3">
                <InputText @bind-Value="Input.Email" id="Input.Email" class="form-control" autocomplete="username" aria-required="true" placeholder="name@example.com" />
                <label for="Input.Email" class="form-label">Email</label>
                <ValidationMessage For="() => Input.Email" class="text-danger" />
            </div>
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.Password" id="Input.Password" class="form-control" autocomplete="new-password" aria-required="true" placeholder="Please enter your password." />
                <label for="Input.Password" class="form-label">Password</label>
                <ValidationMessage For="() => Input.Password" class="text-danger" />
            </div>
            <div class="form-floating mb-3">
                <InputText type="password" @bind-Value="Input.ConfirmPassword" id="Input.ConfirmPassword" class="form-control" autocomplete="new-password" aria-required="true" placeholder="Please confirm your password." />
                <label for="Input.ConfirmPassword" class="form-label">Confirm password</label>
                <ValidationMessage For="() => Input.ConfirmPassword" class="text-danger" />
            </div>
            <button type="submit" class="w-100 btn btn-lg btn-primary">Reset</button>
        </EditForm>
    </div>
</div>

@code {
    private IEnumerable<IdentityError>? identityErrors;

    [SupplyParameterFromForm]
    private InputModel Input { get; set; } = default!;

    [SupplyParameterFromQuery]
    private string? Code { get; set; }

    private string? Message => identityErrors is null ? null : $"Error: {string.Join(", ", identityErrors.Select(error => error.Description))}";

    protected override void OnInitialized()
    {
        Input ??= new();

        if (Code is null)
        {
            RedirectManager.RedirectTo("Account/InvalidPasswordReset");
            return;
        }

        Input.Code = Encoding.UTF8.GetString(WebEncoders.Base64UrlDecode(Code));
    }

    private async Task OnValidSubmitAsync()
    {
        var user = await UserManager.FindByEmailAsync(Input.Email);
        if (user is null)
        {
            // Don't reveal that the user does not exist
            RedirectManager.RedirectTo("Account/ResetPasswordConfirmation");
            return;
        }

        var result = await UserManager.ResetPasswordAsync(user, Input.Code, Input.Password);
        if (result.Succeeded)
        {
            RedirectManager.RedirectTo("Account/ResetPasswordConfirmation");
            return;
        }

        identityErrors = result.Errors;
    }

    private sealed class InputModel
    {
        [Required]
        [EmailAddress]
        public string Email { get; set; } = "";

        [Required]
        [StringLength(100, ErrorMessage = "The {0} must be at least {2} and at max {1} characters long.", MinimumLength = 6)]
        [DataType(DataType.Password)]
        public string Password { get; set; } = "";

        [DataType(DataType.Password)]
        [Display(Name = "Confirm password")]
        [Compare("Password", ErrorMessage = "The password and confirmation password do not match.")]
        public string ConfirmPassword { get; set; } = "";

        [Required]
        public string Code { get; set; } = "";
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/ResetPasswordConfirmation.razor">
﻿@page "/Account/ResetPasswordConfirmation"
<PageTitle>Reset password confirmation</PageTitle>

<h1>Reset password confirmation</h1>
<p role="alert">
    Your password has been reset. Please <a href="Account/Login">click here to log in</a>.
</p>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Pages/_Imports.razor">
﻿@using HickoryTodos.Components.Account.Shared
@attribute [ExcludeFromInteractiveRouting]
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/PasskeyInputModel.cs">
namespace HickoryTodos.Components.Account;

public class PasskeyInputModel
{
    public string? CredentialJson { get; set; }
    public string? Error { get; set; }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/PasskeyOperation.cs">
namespace HickoryTodos.Components.Account;

public enum PasskeyOperation
{
    Create = 0,
    Request = 1,
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Shared/ExternalLoginPicker.razor">
﻿@using Microsoft.AspNetCore.Authentication
@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject SignInManager<ApplicationUser> SignInManager
@inject IdentityRedirectManager RedirectManager

@if (externalLogins.Length == 0)
{
    <div>
        <p>
            There are no external authentication services configured. See this <a href="https://go.microsoft.com/fwlink/?LinkID=532715">article
            about setting up this ASP.NET application to support logging in via external services</a>.
        </p>
    </div>
}
else
{
    <form class="form-horizontal" action="Account/PerformExternalLogin" method="post">
        <div>
            <AntiforgeryToken />
            <input type="hidden" name="ReturnUrl" value="@ReturnUrl" />
            <p>
                @foreach (var provider in externalLogins)
                {
                    <button type="submit" class="btn btn-primary" name="provider" value="@provider.Name" title="Log in using your @provider.DisplayName account">@provider.DisplayName</button>
                }
            </p>
        </div>
    </form>
}

@code {
    private AuthenticationScheme[] externalLogins = [];

    [SupplyParameterFromQuery]
    private string? ReturnUrl { get; set; }

    protected override async Task OnInitializedAsync()
    {
        externalLogins = (await SignInManager.GetExternalAuthenticationSchemesAsync()).ToArray();
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Shared/ManageLayout.razor">
﻿@inherits LayoutComponentBase
@layout HickoryTodos.Components.Layout.MainLayout

<h1>Manage your account</h1>

<div>
    <h2>Change your account settings</h2>
    <hr />
    <div class="row">
        <div class="col-lg-3">
            <ManageNavMenu />
        </div>
        <div class="col-lg-9">
            @Body
        </div>
    </div>
</div>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Shared/ManageNavMenu.razor">
﻿@using Microsoft.AspNetCore.Identity
@using HickoryTodos.Data

@inject SignInManager<ApplicationUser> SignInManager

<ul class="nav nav-pills flex-column">
    <li class="nav-item">
        <NavLink class="nav-link" href="Account/Manage" Match="NavLinkMatch.All">Profile</NavLink>
    </li>
    <li class="nav-item">
        <NavLink class="nav-link" href="Account/Manage/Email">Email</NavLink>
    </li>
    <li class="nav-item">
        <NavLink class="nav-link" href="Account/Manage/ChangePassword">Password</NavLink>
    </li>
    @if (hasExternalLogins)
    {
        <li class="nav-item">
            <NavLink class="nav-link" href="Account/Manage/ExternalLogins">External logins</NavLink>
        </li>
    }
    <li class="nav-item">
        <NavLink class="nav-link" href="Account/Manage/TwoFactorAuthentication">Two-factor authentication</NavLink>
    </li>
    <li class="nav-item">
        <NavLink class="nav-link" href="Account/Manage/Passkeys">Passkeys</NavLink>
    </li>
    <li class="nav-item">
        <NavLink class="nav-link" href="Account/Manage/PersonalData">Personal data</NavLink>
    </li>
</ul>

@code {
    private bool hasExternalLogins;

    protected override async Task OnInitializedAsync()
    {
        hasExternalLogins = (await SignInManager.GetExternalAuthenticationSchemesAsync()).Any();
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Shared/PasskeySubmit.razor">
﻿@using Microsoft.AspNetCore.Antiforgery
@inject IServiceProvider Services

<button type="submit" name="__passkeySubmit" @attributes="AdditionalAttributes">@ChildContent</button>
<passkey-submit
    operation="@Operation"
    name="@Name"
    email-name="@EmailName"
    request-token-name="@tokens?.HeaderName"
    request-token-value="@tokens?.RequestToken">
</passkey-submit>

@code {
    private AntiforgeryTokenSet? tokens;

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    [Parameter]
    [EditorRequired]
    public PasskeyOperation Operation { get; set; }

    [Parameter]
    [EditorRequired]
    public string Name { get; set; } = default!;

    [Parameter]
    public string? EmailName { get; set; }

    [Parameter]
    public RenderFragment? ChildContent { get; set; }

    [Parameter(CaptureUnmatchedValues = true)]
    public IDictionary<string, object>? AdditionalAttributes { get; set; }

    protected override void OnInitialized()
    {
        tokens = Services.GetService<IAntiforgery>()?.GetTokens(HttpContext);
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Shared/PasskeySubmit.razor.js">
﻿const browserSupportsPasskeys =
    typeof navigator.credentials !== 'undefined' &&
    typeof window.PublicKeyCredential !== 'undefined' &&
    typeof window.PublicKeyCredential.parseCreationOptionsFromJSON === 'function' &&
    typeof window.PublicKeyCredential.parseRequestOptionsFromJSON === 'function';

async function fetchWithErrorHandling(url, options = {}) {
    const response = await fetch(url, {
        credentials: 'include',
        ...options
    });
    if (!response.ok) {
        const text = await response.text();
        console.error(text);
        throw new Error(`The server responded with status ${response.status}.`);
    }
    return response;
}

async function createCredential(headers, signal) {
    const optionsResponse = await fetchWithErrorHandling('/Account/PasskeyCreationOptions', {
        method: 'POST',
        headers,
        signal,
    });
    const optionsJson = await optionsResponse.json();
    const options = PublicKeyCredential.parseCreationOptionsFromJSON(optionsJson);
    return await navigator.credentials.create({ publicKey: options, signal });
}

async function requestCredential(email, mediation, headers, signal) {
    const optionsResponse = await fetchWithErrorHandling(`/Account/PasskeyRequestOptions?username=${email}`, {
        method: 'POST',
        headers,
        signal,
    });
    const optionsJson = await optionsResponse.json();
    const options = PublicKeyCredential.parseRequestOptionsFromJSON(optionsJson);
    return await navigator.credentials.get({ publicKey: options, mediation, signal });
}

customElements.define('passkey-submit', class extends HTMLElement {
    static formAssociated = true;

    connectedCallback() {
        this.internals = this.attachInternals();
        this.attrs = {
            operation: this.getAttribute('operation'),
            name: this.getAttribute('name'),
            emailName: this.getAttribute('email-name'),
            requestTokenName: this.getAttribute('request-token-name'),
            requestTokenValue: this.getAttribute('request-token-value'),
        };

        this.internals.form.addEventListener('submit', (event) => {
            if (event.submitter?.name === '__passkeySubmit') {
                event.preventDefault();
                this.obtainAndSubmitCredential();
            }
        });

        this.tryAutofillPasskey();
    }

    disconnectedCallback() {
        this.abortController?.abort();
    }

    async obtainCredential(useConditionalMediation, signal) {
        if (!browserSupportsPasskeys) {
            throw new Error('Some passkey features are missing. Please update your browser.');
        }

        const headers = {
            [this.attrs.requestTokenName]: this.attrs.requestTokenValue,
        };

        if (this.attrs.operation === 'Create') {
            return await createCredential(headers, signal);
        } else if (this.attrs.operation === 'Request') {
            const email = new FormData(this.internals.form).get(this.attrs.emailName);
            const mediation = useConditionalMediation ? 'conditional' : undefined;
            return await requestCredential(email, mediation, headers, signal);
        } else {
            throw new Error(`Unknown passkey operation '${this.attrs.operation}'.`);
        }
    }

    async obtainAndSubmitCredential(useConditionalMediation = false) {
        this.abortController?.abort();
        this.abortController = new AbortController();
        const signal = this.abortController.signal;
        const formData = new FormData();
        try {
            const credential = await this.obtainCredential(useConditionalMediation, signal);
            const credentialJson = JSON.stringify(credential);
            formData.append(`${this.attrs.name}.CredentialJson`, credentialJson);
        } catch (error) {
            if (error.name === 'AbortError') {
                // The user explicitly canceled the operation - return without error.
                return;
            }
            console.error(error);
            if (useConditionalMediation) {
                // An error occurred during conditional mediation, which is not user-initiated.
                // We log the error in the console but do not relay it to the user.
                return;
            }
            const errorMessage = error.name === 'NotAllowedError'
                ? 'No passkey was provided by the authenticator.'
                : error.message;
            formData.append(`${this.attrs.name}.Error`, errorMessage);
        }
        this.internals.setFormValue(formData);
        this.internals.form.submit();
    }

    async tryAutofillPasskey() {
        if (browserSupportsPasskeys && this.attrs.operation === 'Request' && await PublicKeyCredential.isConditionalMediationAvailable?.()) {
            await this.obtainAndSubmitCredential(/* useConditionalMediation */ true);
        }
    }
});
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Shared/RedirectToLogin.razor">
﻿@inject NavigationManager NavigationManager

@code {
    protected override void OnInitialized()
    {
        NavigationManager.NavigateTo($"Account/Login?returnUrl={Uri.EscapeDataString(NavigationManager.Uri)}", forceLoad: true);
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Shared/ShowRecoveryCodes.razor">
﻿<StatusMessage Message="@StatusMessage" />
<h3>Recovery codes</h3>
<div class="alert alert-warning" role="alert">
    <p>
        <strong>Put these codes in a safe place.</strong>
    </p>
    <p>
        If you lose your device and don't have the recovery codes you will lose access to your account.
    </p>
</div>
<div class="row">
    <div class="col-md-12">
        @foreach (var recoveryCode in RecoveryCodes)
        {
            <div>
                <code class="recovery-code">@recoveryCode</code>
            </div>
        }
    </div>
</div>

@code {
    [Parameter]
    public string[] RecoveryCodes { get; set; } = [];

    [Parameter]
    public string? StatusMessage { get; set; }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Account/Shared/StatusMessage.razor">
﻿@if (!string.IsNullOrEmpty(DisplayMessage))
{
    var statusMessageClass = DisplayMessage.StartsWith("Error") ? "danger" : "success";
    <div class="alert alert-@statusMessageClass" role="alert">
        @DisplayMessage
    </div>
}

@code {
    private string? messageFromCookie;

    [Parameter]
    public string? Message { get; set; }

    [CascadingParameter]
    private HttpContext HttpContext { get; set; } = default!;

    private string? DisplayMessage => Message ?? messageFromCookie;

    protected override void OnInitialized()
    {
        messageFromCookie = HttpContext.Request.Cookies[IdentityRedirectManager.StatusCookieName];

        if (messageFromCookie is not null)
        {
            HttpContext.Response.Cookies.Delete(IdentityRedirectManager.StatusCookieName);
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/App.razor">
﻿<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <base href="/" />
    <ResourcePreloader />
    <link rel="stylesheet" href="@Assets["app.css"]" />
    <link rel="stylesheet" href="@Assets["HickoryTodos.styles.css"]" />
    <ImportMap />
    <HeadOutlet />
</head>

<body>
    <Routes />
    <ReconnectModal />
    <script src="@Assets["_framework/blazor.web.js"]"></script>
    <script src="@Assets["Components/Account/Shared/PasskeySubmit.razor.js"]" type="module"></script>
</body>

</html>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Layout/MainLayout.razor">
﻿@inherits LayoutComponentBase

@Body

<div id="blazor-error-ui" data-nosnippet>
    An unhandled error has occurred.
    <a href="." class="reload">Reload</a>
    <span class="dismiss">🗙</span>
</div>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Layout/MainLayout.razor.css">
#blazor-error-ui {
    color-scheme: light only;
    background: lightyellow;
    bottom: 0;
    box-shadow: 0 -1px 2px rgba(0, 0, 0, 0.2);
    box-sizing: border-box;
    display: none;
    left: 0;
    padding: 0.6rem 1.25rem 0.7rem 1.25rem;
    position: fixed;
    width: 100%;
    z-index: 1000;
}

    #blazor-error-ui .dismiss {
        cursor: pointer;
        position: absolute;
        right: 0.75rem;
        top: 0.5rem;
    }
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Layout/ReconnectModal.razor">
﻿<script type="module" src="@Assets["Components/Layout/ReconnectModal.razor.js"]"></script>

<dialog id="components-reconnect-modal" data-nosnippet>
    <div class="components-reconnect-container">
        <div class="components-rejoining-animation" aria-hidden="true">
            <div></div>
            <div></div>
        </div>
        <p class="components-reconnect-first-attempt-visible">
            Rejoining the server...
        </p>
        <p class="components-reconnect-repeated-attempt-visible">
            Rejoin failed... trying again in <span id="components-seconds-to-next-attempt"></span> seconds.
        </p>
        <p class="components-reconnect-failed-visible">
            Failed to rejoin.<br />Please retry or reload the page.
        </p>
        <button id="components-reconnect-button" class="components-reconnect-failed-visible">
            Retry
        </button>
        <p class="components-pause-visible">
            The session has been paused by the server.
        </p>
        <p class="components-resume-failed-visible">
            Failed to resume the session.<br />Please retry or reload the page.
        </p>
        <button id="components-resume-button" class="components-pause-visible components-resume-failed-visible">
            Resume
        </button>
    </div>
</dialog>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Layout/ReconnectModal.razor.css">
.components-reconnect-first-attempt-visible,
.components-reconnect-repeated-attempt-visible,
.components-reconnect-failed-visible,
.components-pause-visible,
.components-resume-failed-visible,
.components-rejoining-animation {
    display: none;
}

#components-reconnect-modal.components-reconnect-show .components-reconnect-first-attempt-visible,
#components-reconnect-modal.components-reconnect-show .components-rejoining-animation,
#components-reconnect-modal.components-reconnect-paused .components-pause-visible,
#components-reconnect-modal.components-reconnect-resume-failed .components-resume-failed-visible,
#components-reconnect-modal.components-reconnect-retrying,
#components-reconnect-modal.components-reconnect-retrying .components-reconnect-repeated-attempt-visible,
#components-reconnect-modal.components-reconnect-retrying .components-rejoining-animation,
#components-reconnect-modal.components-reconnect-failed,
#components-reconnect-modal.components-reconnect-failed .components-reconnect-failed-visible {
    display: block;
}


#components-reconnect-modal {
    background-color: white;
    width: 20rem;
    margin: 20vh auto;
    padding: 2rem;
    border: 0;
    border-radius: 0.5rem;
    box-shadow: 0 3px 6px 2px rgba(0, 0, 0, 0.3);
    opacity: 0;
    transition: display 0.5s allow-discrete, overlay 0.5s allow-discrete;
    animation: components-reconnect-modal-fadeOutOpacity 0.5s both;
    &[open]

{
    animation: components-reconnect-modal-slideUp 1.5s cubic-bezier(.05, .89, .25, 1.02) 0.3s, components-reconnect-modal-fadeInOpacity 0.5s ease-in-out 0.3s;
    animation-fill-mode: both;
}

}

#components-reconnect-modal::backdrop {
    background-color: rgba(0, 0, 0, 0.4);
    animation: components-reconnect-modal-fadeInOpacity 0.5s ease-in-out;
    opacity: 1;
}

@keyframes components-reconnect-modal-slideUp {
    0% {
        transform: translateY(30px) scale(0.95);
    }

    100% {
        transform: translateY(0);
    }
}

@keyframes components-reconnect-modal-fadeInOpacity {
    0% {
        opacity: 0;
    }

    100% {
        opacity: 1;
    }
}

@keyframes components-reconnect-modal-fadeOutOpacity {
    0% {
        opacity: 1;
    }

    100% {
        opacity: 0;
    }
}

.components-reconnect-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
}

#components-reconnect-modal p {
    margin: 0;
    text-align: center;
}

#components-reconnect-modal button {
    border: 0;
    background-color: #6b9ed2;
    color: white;
    padding: 4px 24px;
    border-radius: 4px;
}

    #components-reconnect-modal button:hover {
        background-color: #3b6ea2;
    }

    #components-reconnect-modal button:active {
        background-color: #6b9ed2;
    }

.components-rejoining-animation {
    position: relative;
    width: 80px;
    height: 80px;
}

    .components-rejoining-animation div {
        position: absolute;
        border: 3px solid #0087ff;
        opacity: 1;
        border-radius: 50%;
        animation: components-rejoining-animation 1.5s cubic-bezier(0, 0.2, 0.8, 1) infinite;
    }

        .components-rejoining-animation div:nth-child(2) {
            animation-delay: -0.5s;
        }

@keyframes components-rejoining-animation {
    0% {
        top: 40px;
        left: 40px;
        width: 0;
        height: 0;
        opacity: 0;
    }

    4.9% {
        top: 40px;
        left: 40px;
        width: 0;
        height: 0;
        opacity: 0;
    }

    5% {
        top: 40px;
        left: 40px;
        width: 0;
        height: 0;
        opacity: 1;
    }

    100% {
        top: 0px;
        left: 0px;
        width: 80px;
        height: 80px;
        opacity: 0;
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Layout/ReconnectModal.razor.js">
// Set up event handlers
const reconnectModal = document.getElementById("components-reconnect-modal");
reconnectModal.addEventListener("components-reconnect-state-changed", handleReconnectStateChanged);

const retryButton = document.getElementById("components-reconnect-button");
retryButton.addEventListener("click", retry);

const resumeButton = document.getElementById("components-resume-button");
resumeButton.addEventListener("click", resume);

function handleReconnectStateChanged(event) {
    if (event.detail.state === "show") {
        reconnectModal.showModal();
    } else if (event.detail.state === "hide") {
        reconnectModal.close();
    } else if (event.detail.state === "failed") {
        document.addEventListener("visibilitychange", retryWhenDocumentBecomesVisible);
    } else if (event.detail.state === "rejected") {
        location.reload();
    }
}

async function retry() {
    document.removeEventListener("visibilitychange", retryWhenDocumentBecomesVisible);

    try {
        // Reconnect will asynchronously return:
        // - true to mean success
        // - false to mean we reached the server, but it rejected the connection (e.g., unknown circuit ID)
        // - exception to mean we didn't reach the server (this can be sync or async)
        const successful = await Blazor.reconnect();
        if (!successful) {
            // We have been able to reach the server, but the circuit is no longer available.
            // We'll reload the page so the user can continue using the app as quickly as possible.
            const resumeSuccessful = await Blazor.resumeCircuit();
            if (!resumeSuccessful) {
                location.reload();
            } else {
                reconnectModal.close();
            }
        }
    } catch (err) {
        // We got an exception, server is currently unavailable
        document.addEventListener("visibilitychange", retryWhenDocumentBecomesVisible);
    }
}

async function resume() {
    try {
        const successful = await Blazor.resumeCircuit();
        if (!successful) {
            location.reload();
        }
    } catch {
        reconnectModal.classList.replace("components-reconnect-paused", "components-reconnect-resume-failed");
    }
}

async function retryWhenDocumentBecomesVisible() {
    if (document.visibilityState === "visible") {
        await retry();
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Pages/Error.razor">
﻿@page "/Error"
@using System.Diagnostics

<PageTitle>Error</PageTitle>

<h1 class="text-danger">Error.</h1>
<h2 class="text-danger">An error occurred while processing your request.</h2>

@if (ShowRequestId)
{
    <p>
        <strong>Request ID:</strong> <code>@RequestId</code>
    </p>
}

<h3>Development Mode</h3>
<p>
    Swapping to <strong>Development</strong> environment will display more detailed information about the error that occurred.
</p>
<p>
    <strong>The Development environment shouldn't be enabled for deployed applications.</strong>
    It can result in displaying sensitive information from exceptions to end users.
    For local debugging, enable the <strong>Development</strong> environment by setting the <strong>ASPNETCORE_ENVIRONMENT</strong> environment variable to <strong>Development</strong>
    and restarting the app.
</p>

@code{
    [CascadingParameter]
    private HttpContext? HttpContext { get; set; }

    private string? RequestId { get; set; }
    private bool ShowRequestId => !string.IsNullOrEmpty(RequestId);

    protected override void OnInitialized() =>
        RequestId = Activity.Current?.Id ?? HttpContext?.TraceIdentifier;
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Pages/Home.razor">
﻿@page "/"

<PageTitle>Home</PageTitle>

<h1>Hello, world!</h1>

Welcome to your new app.
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Pages/NotFound.razor">
﻿@page "/not-found"
@layout MainLayout

<h3>Not Found</h3>
<p>Sorry, the content you are looking for does not exist.</p></demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/Routes.razor">
﻿@using HickoryTodos.Components.Account.Shared
<Router AppAssembly="typeof(Program).Assembly" NotFoundPage="typeof(Pages.NotFound)">
    <Found Context="routeData">
        <AuthorizeRouteView RouteData="routeData" DefaultLayout="typeof(Layout.MainLayout)">
            <NotAuthorized>
                <RedirectToLogin />
            </NotAuthorized>
        </AuthorizeRouteView>
        <FocusOnNavigate RouteData="routeData" Selector="h1" />
    </Found>
</Router>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Components/_Imports.razor">
﻿@using System.Net.Http
@using System.Net.Http.Json
@using Microsoft.AspNetCore.Components.Authorization
@using Microsoft.AspNetCore.Components.Forms
@using Microsoft.AspNetCore.Components.Routing
@using Microsoft.AspNetCore.Components.Web
@using static Microsoft.AspNetCore.Components.Web.RenderMode
@using Microsoft.AspNetCore.Components.Web.Virtualization
@using Microsoft.JSInterop
@using HickoryTodos
@using HickoryTodos.Components
@using HickoryTodos.Components.Layout
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Data/ApplicationDbContext.cs">
using Microsoft.AspNetCore.Identity.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore;

namespace HickoryTodos.Data;

public class ApplicationDbContext(DbContextOptions<ApplicationDbContext> options) : IdentityDbContext<ApplicationUser>(options)
{
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Data/ApplicationUser.cs">
using Microsoft.AspNetCore.Identity;

namespace HickoryTodos.Data;

// Add profile data for application users by adding properties to the ApplicationUser class
public class ApplicationUser : IdentityUser
{
}

</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Data/Migrations/00000000000000_CreateIdentitySchema.Designer.cs">
// <auto-generated />
using System;
using HickoryTodos.Data;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;

#nullable disable

namespace HickoryTodos.Migrations
{
    [DbContext(typeof(ApplicationDbContext))]
    [Migration("00000000000000_CreateIdentitySchema")]
    partial class CreateIdentitySchema
    {
        /// <inheritdoc />
        protected override void BuildTargetModel(ModelBuilder modelBuilder)
        {
#pragma warning disable 612, 618
            modelBuilder.HasAnnotation("ProductVersion", "10.0.0");

            modelBuilder.Entity("HickoryTodos.Data.ApplicationUser", b =>
                {
                    b.Property<string>("Id")
                        .HasColumnType("TEXT");

                    b.Property<int>("AccessFailedCount")
                        .HasColumnType("INTEGER");

                    b.Property<string>("ConcurrencyStamp")
                        .IsConcurrencyToken()
                        .HasColumnType("TEXT");

                    b.Property<string>("Email")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<bool>("EmailConfirmed")
                        .HasColumnType("INTEGER");

                    b.Property<bool>("LockoutEnabled")
                        .HasColumnType("INTEGER");

                    b.Property<DateTimeOffset?>("LockoutEnd")
                        .HasColumnType("TEXT");

                    b.Property<string>("NormalizedEmail")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<string>("NormalizedUserName")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<string>("PasswordHash")
                        .HasColumnType("TEXT");

                    b.Property<string>("PhoneNumber")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<bool>("PhoneNumberConfirmed")
                        .HasColumnType("INTEGER");

                    b.Property<string>("SecurityStamp")
                        .HasColumnType("TEXT");

                    b.Property<bool>("TwoFactorEnabled")
                        .HasColumnType("INTEGER");

                    b.Property<string>("UserName")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.HasKey("Id");

                    b.HasIndex("NormalizedEmail")
                        .HasDatabaseName("EmailIndex");

                    b.HasIndex("NormalizedUserName")
                        .IsUnique()
                        .HasDatabaseName("UserNameIndex");

                    b.ToTable("AspNetUsers", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityRole", b =>
                {
                    b.Property<string>("Id")
                        .HasColumnType("TEXT");

                    b.Property<string>("ConcurrencyStamp")
                        .IsConcurrencyToken()
                        .HasColumnType("TEXT");

                    b.Property<string>("Name")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<string>("NormalizedName")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.HasKey("Id");

                    b.HasIndex("NormalizedName")
                        .IsUnique()
                        .HasDatabaseName("RoleNameIndex");

                    b.ToTable("AspNetRoles", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityRoleClaim<string>", b =>
                {
                    b.Property<int>("Id")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("INTEGER");

                    b.Property<string>("ClaimType")
                        .HasColumnType("TEXT");

                    b.Property<string>("ClaimValue")
                        .HasColumnType("TEXT");

                    b.Property<string>("RoleId")
                        .IsRequired()
                        .HasColumnType("TEXT");

                    b.HasKey("Id");

                    b.HasIndex("RoleId");

                    b.ToTable("AspNetRoleClaims", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserClaim<string>", b =>
                {
                    b.Property<int>("Id")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("INTEGER");

                    b.Property<string>("ClaimType")
                        .HasColumnType("TEXT");

                    b.Property<string>("ClaimValue")
                        .HasColumnType("TEXT");

                    b.Property<string>("UserId")
                        .IsRequired()
                        .HasColumnType("TEXT");

                    b.HasKey("Id");

                    b.HasIndex("UserId");

                    b.ToTable("AspNetUserClaims", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserLogin<string>", b =>
                {
                    b.Property<string>("LoginProvider")
                        .HasMaxLength(128)
                        .HasColumnType("TEXT");

                    b.Property<string>("ProviderKey")
                        .HasMaxLength(128)
                        .HasColumnType("TEXT");

                    b.Property<string>("ProviderDisplayName")
                        .HasColumnType("TEXT");

                    b.Property<string>("UserId")
                        .IsRequired()
                        .HasColumnType("TEXT");

                    b.HasKey("LoginProvider", "ProviderKey");

                    b.HasIndex("UserId");

                    b.ToTable("AspNetUserLogins", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserPasskey<string>", b =>
                {
                    b.Property<byte[]>("CredentialId")
                        .HasMaxLength(1024)
                        .HasColumnType("BLOB");

                    b.Property<string>("UserId")
                        .IsRequired()
                        .HasColumnType("TEXT");

                    b.HasKey("CredentialId");

                    b.HasIndex("UserId");

                    b.ToTable("AspNetUserPasskeys", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserRole<string>", b =>
                {
                    b.Property<string>("UserId")
                        .HasColumnType("TEXT");

                    b.Property<string>("RoleId")
                        .HasColumnType("TEXT");

                    b.HasKey("UserId", "RoleId");

                    b.HasIndex("RoleId");

                    b.ToTable("AspNetUserRoles", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserToken<string>", b =>
                {
                    b.Property<string>("UserId")
                        .HasColumnType("TEXT");

                    b.Property<string>("LoginProvider")
                        .HasMaxLength(128)
                        .HasColumnType("TEXT");

                    b.Property<string>("Name")
                        .HasMaxLength(128)
                        .HasColumnType("TEXT");

                    b.Property<string>("Value")
                        .HasColumnType("TEXT");

                    b.HasKey("UserId", "LoginProvider", "Name");

                    b.ToTable("AspNetUserTokens", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityRoleClaim<string>", b =>
                {
                    b.HasOne("Microsoft.AspNetCore.Identity.IdentityRole", null)
                        .WithMany()
                        .HasForeignKey("RoleId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserClaim<string>", b =>
                {
                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserLogin<string>", b =>
                {
                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserPasskey<string>", b =>
                {
                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();

                    b.OwnsOne("Microsoft.AspNetCore.Identity.IdentityPasskeyData", "Data", b1 =>
                        {
                            b1.Property<byte[]>("IdentityUserPasskeyCredentialId")
                                .HasColumnType("BLOB");

                            b1.Property<byte[]>("AttestationObject")
                                .IsRequired()
                                .HasColumnType("BLOB");

                            b1.Property<byte[]>("ClientDataJson")
                                .IsRequired()
                                .HasColumnType("BLOB");

                            b1.Property<DateTimeOffset>("CreatedAt")
                                .HasColumnType("TEXT");

                            b1.Property<bool>("IsBackedUp")
                                .HasColumnType("INTEGER");

                            b1.Property<bool>("IsBackupEligible")
                                .HasColumnType("INTEGER");

                            b1.Property<bool>("IsUserVerified")
                                .HasColumnType("INTEGER");

                            b1.Property<string>("Name")
                                .HasColumnType("TEXT");

                            b1.Property<byte[]>("PublicKey")
                                .IsRequired()
                                .HasColumnType("BLOB");

                            b1.Property<uint>("SignCount")
                                .HasColumnType("INTEGER");

                            b1.PrimitiveCollection<string>("Transports")
                                .HasColumnType("TEXT");

                            b1.HasKey("IdentityUserPasskeyCredentialId");

                            b1.ToTable("AspNetUserPasskeys");

                            b1.ToJson("Data");

                            b1.WithOwner()
                                .HasForeignKey("IdentityUserPasskeyCredentialId");
                        });

                    b.Navigation("Data")
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserRole<string>", b =>
                {
                    b.HasOne("Microsoft.AspNetCore.Identity.IdentityRole", null)
                        .WithMany()
                        .HasForeignKey("RoleId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();

                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserToken<string>", b =>
                {
                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });
#pragma warning restore 612, 618
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Data/Migrations/00000000000000_CreateIdentitySchema.cs">
﻿using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace HickoryTodos.Migrations
{
    /// <inheritdoc />
    public partial class CreateIdentitySchema : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "AspNetRoles",
                columns: table => new
                {
                    Id = table.Column<string>(type: "TEXT", nullable: false),
                    Name = table.Column<string>(type: "TEXT", maxLength: 256, nullable: true),
                    NormalizedName = table.Column<string>(type: "TEXT", maxLength: 256, nullable: true),
                    ConcurrencyStamp = table.Column<string>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AspNetRoles", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "AspNetUsers",
                columns: table => new
                {
                    Id = table.Column<string>(type: "TEXT", nullable: false),
                    UserName = table.Column<string>(type: "TEXT", maxLength: 256, nullable: true),
                    NormalizedUserName = table.Column<string>(type: "TEXT", maxLength: 256, nullable: true),
                    Email = table.Column<string>(type: "TEXT", maxLength: 256, nullable: true),
                    NormalizedEmail = table.Column<string>(type: "TEXT", maxLength: 256, nullable: true),
                    EmailConfirmed = table.Column<bool>(type: "INTEGER", nullable: false),
                    PasswordHash = table.Column<string>(type: "TEXT", nullable: true),
                    SecurityStamp = table.Column<string>(type: "TEXT", nullable: true),
                    ConcurrencyStamp = table.Column<string>(type: "TEXT", nullable: true),
                    PhoneNumber = table.Column<string>(type: "TEXT", maxLength: 256, nullable: true),
                    PhoneNumberConfirmed = table.Column<bool>(type: "INTEGER", nullable: false),
                    TwoFactorEnabled = table.Column<bool>(type: "INTEGER", nullable: false),
                    LockoutEnd = table.Column<DateTimeOffset>(type: "TEXT", nullable: true),
                    LockoutEnabled = table.Column<bool>(type: "INTEGER", nullable: false),
                    AccessFailedCount = table.Column<int>(type: "INTEGER", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AspNetUsers", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "AspNetRoleClaims",
                columns: table => new
                {
                    Id = table.Column<int>(type: "INTEGER", nullable: false)
                        .Annotation("Sqlite:Autoincrement", true),
                    RoleId = table.Column<string>(type: "TEXT", nullable: false),
                    ClaimType = table.Column<string>(type: "TEXT", nullable: true),
                    ClaimValue = table.Column<string>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AspNetRoleClaims", x => x.Id);
                    table.ForeignKey(
                        name: "FK_AspNetRoleClaims_AspNetRoles_RoleId",
                        column: x => x.RoleId,
                        principalTable: "AspNetRoles",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "AspNetUserClaims",
                columns: table => new
                {
                    Id = table.Column<int>(type: "INTEGER", nullable: false)
                        .Annotation("Sqlite:Autoincrement", true),
                    UserId = table.Column<string>(type: "TEXT", nullable: false),
                    ClaimType = table.Column<string>(type: "TEXT", nullable: true),
                    ClaimValue = table.Column<string>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AspNetUserClaims", x => x.Id);
                    table.ForeignKey(
                        name: "FK_AspNetUserClaims_AspNetUsers_UserId",
                        column: x => x.UserId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "AspNetUserLogins",
                columns: table => new
                {
                    LoginProvider = table.Column<string>(type: "TEXT", maxLength: 128, nullable: false),
                    ProviderKey = table.Column<string>(type: "TEXT", maxLength: 128, nullable: false),
                    ProviderDisplayName = table.Column<string>(type: "TEXT", nullable: true),
                    UserId = table.Column<string>(type: "TEXT", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AspNetUserLogins", x => new { x.LoginProvider, x.ProviderKey });
                    table.ForeignKey(
                        name: "FK_AspNetUserLogins_AspNetUsers_UserId",
                        column: x => x.UserId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "AspNetUserPasskeys",
                columns: table => new
                {
                    CredentialId = table.Column<byte[]>(type: "BLOB", maxLength: 1024, nullable: false),
                    UserId = table.Column<string>(type: "TEXT", nullable: false),
                    Data = table.Column<string>(type: "TEXT", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AspNetUserPasskeys", x => x.CredentialId);
                    table.ForeignKey(
                        name: "FK_AspNetUserPasskeys_AspNetUsers_UserId",
                        column: x => x.UserId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "AspNetUserRoles",
                columns: table => new
                {
                    UserId = table.Column<string>(type: "TEXT", nullable: false),
                    RoleId = table.Column<string>(type: "TEXT", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AspNetUserRoles", x => new { x.UserId, x.RoleId });
                    table.ForeignKey(
                        name: "FK_AspNetUserRoles_AspNetRoles_RoleId",
                        column: x => x.RoleId,
                        principalTable: "AspNetRoles",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "FK_AspNetUserRoles_AspNetUsers_UserId",
                        column: x => x.UserId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "AspNetUserTokens",
                columns: table => new
                {
                    UserId = table.Column<string>(type: "TEXT", nullable: false),
                    LoginProvider = table.Column<string>(type: "TEXT", maxLength: 128, nullable: false),
                    Name = table.Column<string>(type: "TEXT", maxLength: 128, nullable: false),
                    Value = table.Column<string>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AspNetUserTokens", x => new { x.UserId, x.LoginProvider, x.Name });
                    table.ForeignKey(
                        name: "FK_AspNetUserTokens_AspNetUsers_UserId",
                        column: x => x.UserId,
                        principalTable: "AspNetUsers",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "IX_AspNetRoleClaims_RoleId",
                table: "AspNetRoleClaims",
                column: "RoleId");

            migrationBuilder.CreateIndex(
                name: "RoleNameIndex",
                table: "AspNetRoles",
                column: "NormalizedName",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "IX_AspNetUserClaims_UserId",
                table: "AspNetUserClaims",
                column: "UserId");

            migrationBuilder.CreateIndex(
                name: "IX_AspNetUserLogins_UserId",
                table: "AspNetUserLogins",
                column: "UserId");

            migrationBuilder.CreateIndex(
                name: "IX_AspNetUserPasskeys_UserId",
                table: "AspNetUserPasskeys",
                column: "UserId");

            migrationBuilder.CreateIndex(
                name: "IX_AspNetUserRoles_RoleId",
                table: "AspNetUserRoles",
                column: "RoleId");

            migrationBuilder.CreateIndex(
                name: "EmailIndex",
                table: "AspNetUsers",
                column: "NormalizedEmail");

            migrationBuilder.CreateIndex(
                name: "UserNameIndex",
                table: "AspNetUsers",
                column: "NormalizedUserName",
                unique: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "AspNetRoleClaims");

            migrationBuilder.DropTable(
                name: "AspNetUserClaims");

            migrationBuilder.DropTable(
                name: "AspNetUserLogins");

            migrationBuilder.DropTable(
                name: "AspNetUserPasskeys");

            migrationBuilder.DropTable(
                name: "AspNetUserRoles");

            migrationBuilder.DropTable(
                name: "AspNetUserTokens");

            migrationBuilder.DropTable(
                name: "AspNetRoles");

            migrationBuilder.DropTable(
                name: "AspNetUsers");
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Data/Migrations/ApplicationDbContextModelSnapshot.cs">
// <auto-generated />
using System;
using HickoryTodos.Data;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;

#nullable disable

namespace HickoryTodos.Migrations
{
    [DbContext(typeof(ApplicationDbContext))]
    partial class ApplicationDbContextModelSnapshot : ModelSnapshot
    {
        protected override void BuildModel(ModelBuilder modelBuilder)
        {
#pragma warning disable 612, 618
            modelBuilder.HasAnnotation("ProductVersion", "10.0.0");

            modelBuilder.Entity("HickoryTodos.Data.ApplicationUser", b =>
                {
                    b.Property<string>("Id")
                        .HasColumnType("TEXT");

                    b.Property<int>("AccessFailedCount")
                        .HasColumnType("INTEGER");

                    b.Property<string>("ConcurrencyStamp")
                        .IsConcurrencyToken()
                        .HasColumnType("TEXT");

                    b.Property<string>("Email")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<bool>("EmailConfirmed")
                        .HasColumnType("INTEGER");

                    b.Property<bool>("LockoutEnabled")
                        .HasColumnType("INTEGER");

                    b.Property<DateTimeOffset?>("LockoutEnd")
                        .HasColumnType("TEXT");

                    b.Property<string>("NormalizedEmail")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<string>("NormalizedUserName")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<string>("PasswordHash")
                        .HasColumnType("TEXT");

                    b.Property<string>("PhoneNumber")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<bool>("PhoneNumberConfirmed")
                        .HasColumnType("INTEGER");

                    b.Property<string>("SecurityStamp")
                        .HasColumnType("TEXT");

                    b.Property<bool>("TwoFactorEnabled")
                        .HasColumnType("INTEGER");

                    b.Property<string>("UserName")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.HasKey("Id");

                    b.HasIndex("NormalizedEmail")
                        .HasDatabaseName("EmailIndex");

                    b.HasIndex("NormalizedUserName")
                        .IsUnique()
                        .HasDatabaseName("UserNameIndex");

                    b.ToTable("AspNetUsers", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityRole", b =>
                {
                    b.Property<string>("Id")
                        .HasColumnType("TEXT");

                    b.Property<string>("ConcurrencyStamp")
                        .IsConcurrencyToken()
                        .HasColumnType("TEXT");

                    b.Property<string>("Name")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.Property<string>("NormalizedName")
                        .HasMaxLength(256)
                        .HasColumnType("TEXT");

                    b.HasKey("Id");

                    b.HasIndex("NormalizedName")
                        .IsUnique()
                        .HasDatabaseName("RoleNameIndex");

                    b.ToTable("AspNetRoles", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityRoleClaim<string>", b =>
                {
                    b.Property<int>("Id")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("INTEGER");

                    b.Property<string>("ClaimType")
                        .HasColumnType("TEXT");

                    b.Property<string>("ClaimValue")
                        .HasColumnType("TEXT");

                    b.Property<string>("RoleId")
                        .IsRequired()
                        .HasColumnType("TEXT");

                    b.HasKey("Id");

                    b.HasIndex("RoleId");

                    b.ToTable("AspNetRoleClaims", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserClaim<string>", b =>
                {
                    b.Property<int>("Id")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("INTEGER");

                    b.Property<string>("ClaimType")
                        .HasColumnType("TEXT");

                    b.Property<string>("ClaimValue")
                        .HasColumnType("TEXT");

                    b.Property<string>("UserId")
                        .IsRequired()
                        .HasColumnType("TEXT");

                    b.HasKey("Id");

                    b.HasIndex("UserId");

                    b.ToTable("AspNetUserClaims", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserLogin<string>", b =>
                {
                    b.Property<string>("LoginProvider")
                        .HasMaxLength(128)
                        .HasColumnType("TEXT");

                    b.Property<string>("ProviderKey")
                        .HasMaxLength(128)
                        .HasColumnType("TEXT");

                    b.Property<string>("ProviderDisplayName")
                        .HasColumnType("TEXT");

                    b.Property<string>("UserId")
                        .IsRequired()
                        .HasColumnType("TEXT");

                    b.HasKey("LoginProvider", "ProviderKey");

                    b.HasIndex("UserId");

                    b.ToTable("AspNetUserLogins", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserPasskey<string>", b =>
                {
                    b.Property<byte[]>("CredentialId")
                        .HasMaxLength(1024)
                        .HasColumnType("BLOB");

                    b.Property<string>("UserId")
                        .IsRequired()
                        .HasColumnType("TEXT");

                    b.HasKey("CredentialId");

                    b.HasIndex("UserId");

                    b.ToTable("AspNetUserPasskeys", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserRole<string>", b =>
                {
                    b.Property<string>("UserId")
                        .HasColumnType("TEXT");

                    b.Property<string>("RoleId")
                        .HasColumnType("TEXT");

                    b.HasKey("UserId", "RoleId");

                    b.HasIndex("RoleId");

                    b.ToTable("AspNetUserRoles", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserToken<string>", b =>
                {
                    b.Property<string>("UserId")
                        .HasColumnType("TEXT");

                    b.Property<string>("LoginProvider")
                        .HasMaxLength(128)
                        .HasColumnType("TEXT");

                    b.Property<string>("Name")
                        .HasMaxLength(128)
                        .HasColumnType("TEXT");

                    b.Property<string>("Value")
                        .HasColumnType("TEXT");

                    b.HasKey("UserId", "LoginProvider", "Name");

                    b.ToTable("AspNetUserTokens", (string)null);
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityRoleClaim<string>", b =>
                {
                    b.HasOne("Microsoft.AspNetCore.Identity.IdentityRole", null)
                        .WithMany()
                        .HasForeignKey("RoleId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserClaim<string>", b =>
                {
                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserLogin<string>", b =>
                {
                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserPasskey<string>", b =>
                {
                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();

                    b.OwnsOne("Microsoft.AspNetCore.Identity.IdentityPasskeyData", "Data", b1 =>
                        {
                            b1.Property<byte[]>("IdentityUserPasskeyCredentialId")
                                .HasColumnType("BLOB");

                            b1.Property<byte[]>("AttestationObject")
                                .IsRequired()
                                .HasColumnType("BLOB");

                            b1.Property<byte[]>("ClientDataJson")
                                .IsRequired()
                                .HasColumnType("BLOB");

                            b1.Property<DateTimeOffset>("CreatedAt")
                                .HasColumnType("TEXT");

                            b1.Property<bool>("IsBackedUp")
                                .HasColumnType("INTEGER");

                            b1.Property<bool>("IsBackupEligible")
                                .HasColumnType("INTEGER");

                            b1.Property<bool>("IsUserVerified")
                                .HasColumnType("INTEGER");

                            b1.Property<string>("Name")
                                .HasColumnType("TEXT");

                            b1.Property<byte[]>("PublicKey")
                                .IsRequired()
                                .HasColumnType("BLOB");

                            b1.Property<uint>("SignCount")
                                .HasColumnType("INTEGER");

                            b1.PrimitiveCollection<string>("Transports")
                                .HasColumnType("TEXT");

                            b1.HasKey("IdentityUserPasskeyCredentialId");

                            b1.ToTable("AspNetUserPasskeys");

                            b1.ToJson("Data");

                            b1.WithOwner()
                                .HasForeignKey("IdentityUserPasskeyCredentialId");
                        });

                    b.Navigation("Data")
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserRole<string>", b =>
                {
                    b.HasOne("Microsoft.AspNetCore.Identity.IdentityRole", null)
                        .WithMany()
                        .HasForeignKey("RoleId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();

                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });

            modelBuilder.Entity("Microsoft.AspNetCore.Identity.IdentityUserToken<string>", b =>
                {
                    b.HasOne("HickoryTodos.Data.ApplicationUser", null)
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired();
                });
#pragma warning restore 612, 618
        }
    }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/HickoryTodos.csproj">
<Project Sdk="Microsoft.NET.Sdk.Web">

  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
    <UserSecretsId>aspnet-HickoryTodos-3502bff1-f56c-41ad-9592-2d1e8f20ef26</UserSecretsId>
    <BlazorDisableThrowNavigationException>true</BlazorDisableThrowNavigationException>
  </PropertyGroup>

  <ItemGroup>
    <None Update="Data\app.db" CopyToOutputDirectory="PreserveNewest" ExcludeFromSingleFile="true" />
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.AspNetCore.Diagnostics.EntityFrameworkCore" Version="10.0.12" />
    <PackageReference Include="Microsoft.AspNetCore.Identity.EntityFrameworkCore" Version="10.0.12" />
    <PackageReference Include="Microsoft.EntityFrameworkCore.Sqlite" Version="10.0.12" />
    <PackageReference Include="Microsoft.EntityFrameworkCore.Design" Version="10.0.12" PrivateAssets="all" />
    <PackageReference Include="Microsoft.EntityFrameworkCore.Tools" Version="10.0.12" />
  </ItemGroup>

</Project>
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Program.cs">
using Microsoft.AspNetCore.Components.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.EntityFrameworkCore;
using HickoryTodos.Components;
using HickoryTodos.Components.Account;
using HickoryTodos.Data;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
builder.Services.AddRazorComponents()
    .AddInteractiveServerComponents();

builder.Services.AddCascadingAuthenticationState();
builder.Services.AddScoped<IdentityRedirectManager>();
builder.Services.AddScoped<AuthenticationStateProvider, IdentityRevalidatingAuthenticationStateProvider>();

builder.Services.AddAuthentication(options =>
    {
        options.DefaultScheme = IdentityConstants.ApplicationScheme;
        options.DefaultSignInScheme = IdentityConstants.ExternalScheme;
    })
    .AddIdentityCookies();

var connectionString = builder.Configuration.GetConnectionString("DefaultConnection") ?? throw new InvalidOperationException("Connection string 'DefaultConnection' not found.");
builder.Services.AddDbContext<ApplicationDbContext>(options =>
    options.UseSqlite(connectionString));
builder.Services.AddDatabaseDeveloperPageExceptionFilter();

builder.Services.AddIdentityCore<ApplicationUser>(options =>
    {
        options.SignIn.RequireConfirmedAccount = true;
        options.Stores.SchemaVersion = IdentitySchemaVersions.Version3;
    })
    .AddEntityFrameworkStores<ApplicationDbContext>()
    .AddSignInManager()
    .AddDefaultTokenProviders();

builder.Services.AddSingleton<IEmailSender<ApplicationUser>, IdentityNoOpEmailSender>();

var app = builder.Build();

// Configure the HTTP request pipeline.
if (app.Environment.IsDevelopment())
{
    app.UseMigrationsEndPoint();
}
else
{
    app.UseExceptionHandler("/Error", createScopeForErrors: true);
    // The default HSTS value is 30 days. You may want to change this for production scenarios, see https://aka.ms/aspnetcore-hsts.
    app.UseHsts();
}
app.UseStatusCodePagesWithReExecute("/not-found", createScopeForStatusCodePages: true);
app.UseHttpsRedirection();

app.UseAntiforgery();

app.MapStaticAssets();
app.MapRazorComponents<App>()
    .AddInteractiveServerRenderMode();

// Add additional endpoints required by the Identity /Account Razor components.
app.MapAdditionalIdentityEndpoints();

app.Run();
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/Properties/launchSettings.json">
{
  "$schema": "https://json.schemastore.org/launchsettings.json",
    "profiles": {
      "http": {
        "commandName": "Project",
        "dotnetRunMessages": true,
        "launchBrowser": true,
        "applicationUrl": "http://localhost:0",
        "environmentVariables": {
          "ASPNETCORE_ENVIRONMENT": "Development"
        }
      },
      "https": {
        "commandName": "Project",
        "dotnetRunMessages": true,
        "launchBrowser": true,
        "applicationUrl": "https://localhost:0;http://localhost:0",
        "environmentVariables": {
          "ASPNETCORE_ENVIRONMENT": "Development"
        }
      }
    }
  }
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/appsettings.Development.json">
{
  "Logging": {
    "LogLevel": {
      "Default": "Information",
      "Microsoft.AspNetCore": "Warning"
    }
  }
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/appsettings.json">
{
  "ConnectionStrings": {
    "DefaultConnection": "DataSource=Data/app.db;Cache=Shared"
  },
  "Logging": {
    "LogLevel": {
      "Default": "Information",
      "Microsoft.AspNetCore": "Warning"
    }
  },
  "AllowedHosts": "*"
}
</demo:file>
<demo:file path="examples/projects/blazor-todos/app/wwwroot/app.css">
h1:focus {
    outline: none;
}

.valid.modified:not([type=checkbox]) {
    outline: 1px solid #26b050;
}

.invalid {
    outline: 1px solid #e50000;
}

.validation-message {
    color: #e50000;
}

.blazor-error-boundary {
    background: url(data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNTYiIGhlaWdodD0iNDkiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIG92ZXJmbG93PSJoaWRkZW4iPjxkZWZzPjxjbGlwUGF0aCBpZD0iY2xpcDAiPjxyZWN0IHg9IjIzNSIgeT0iNTEiIHdpZHRoPSI1NiIgaGVpZ2h0PSI0OSIvPjwvY2xpcFBhdGg+PC9kZWZzPjxnIGNsaXAtcGF0aD0idXJsKCNjbGlwMCkiIHRyYW5zZm9ybT0idHJhbnNsYXRlKC0yMzUgLTUxKSI+PHBhdGggZD0iTTI2My41MDYgNTFDMjY0LjcxNyA1MSAyNjUuODEzIDUxLjQ4MzcgMjY2LjYwNiA1Mi4yNjU4TDI2Ny4wNTIgNTIuNzk4NyAyNjcuNTM5IDUzLjYyODMgMjkwLjE4NSA5Mi4xODMxIDI5MC41NDUgOTIuNzk1IDI5MC42NTYgOTIuOTk2QzI5MC44NzcgOTMuNTEzIDI5MSA5NC4wODE1IDI5MSA5NC42NzgyIDI5MSA5Ny4wNjUxIDI4OS4wMzggOTkgMjg2LjYxNyA5OUwyNDAuMzgzIDk5QzIzNy45NjMgOTkgMjM2IDk3LjA2NTEgMjM2IDk0LjY3ODIgMjM2IDk0LjM3OTkgMjM2LjAzMSA5NC4wODg2IDIzNi4wODkgOTMuODA3MkwyMzYuMzM4IDkzLjAxNjIgMjM2Ljg1OCA5Mi4xMzE0IDI1OS40NzMgNTMuNjI5NCAyNTkuOTYxIDUyLjc5ODUgMjYwLjQwNyA1Mi4yNjU4QzI2MS4yIDUxLjQ4MzcgMjYyLjI5NiA1MSAyNjMuNTA2IDUxWk0yNjMuNTg2IDY2LjAxODNDMjYwLjczNyA2Ni4wMTgzIDI1OS4zMTMgNjcuMTI0NSAyNTkuMzEzIDY5LjMzNyAyNTkuMzEzIDY5LjYxMDIgMjU5LjMzMiA2OS44NjA4IDI1OS4zNzEgNzAuMDg4N0wyNjEuNzk1IDg0LjAxNjEgMjY1LjM4IDg0LjAxNjEgMjY3LjgyMSA2OS43NDc1QzI2Ny44NiA2OS43MzA5IDI2Ny44NzkgNjkuNTg3NyAyNjcuODc5IDY5LjMxNzkgMjY3Ljg3OSA2Ny4xMTgyIDI2Ni40NDggNjYuMDE4MyAyNjMuNTg2IDY2LjAxODNaTTI2My41NzYgODYuMDU0N0MyNjEuMDQ5IDg2LjA1NDcgMjU5Ljc4NiA4Ny4zMDA1IDI1OS43ODYgODkuNzkyMSAyNTkuNzg2IDkyLjI4MzcgMjYxLjA0OSA5My41Mjk1IDI2My41NzYgOTMuNTI5NSAyNjYuMTE2IDkzLjUyOTUgMjY3LjM4NyA5Mi4yODM3IDI2Ny4zODcgODkuNzkyMSAyNjcuMzg3IDg3LjMwMDUgMjY2LjExNiA4Ni4wNTQ3IDI2My41NzYgODYuMDU0N1oiIGZpbGw9IiNGRkU1MDAiIGZpbGwtcnVsZT0iZXZlbm9kZCIvPjwvZz48L3N2Zz4=) no-repeat 1rem/1.8rem, #b32121;
    padding: 1rem 1rem 1rem 3.7rem;
    color: white;
}

    .blazor-error-boundary::after {
        content: "An error has occurred."
    }

.darker-border-checkbox.form-check-input {
    border-color: #929292;
}

.form-floating > .form-control-plaintext::placeholder, .form-floating > .form-control::placeholder {
    color: var(--bs-secondary-color);
    text-align: end;
}

.form-floating > .form-control-plaintext:focus::placeholder, .form-floating > .form-control:focus::placeholder {
    text-align: start;
}</demo:file>
<demo:file path="examples/projects/blazor-todos/global.json">
{"sdk":{"version":"10.0.100","rollForward":"latestFeature","allowPrerelease":false}}
</demo:file>
</demo:ingested>
</demo:exec>

Hickory captured the text files from the real template. Git ignore rules excluded the
template's empty SQLite file and the SDK's first-run cache. The supplied Identity migration
reconstructs the database schema instead, and the account pages stay folded
with the rest of the scaffold until you need to inspect them.

## Start the demo with a fresh database

Use an ignored local SQLite directory. In Development apply migrations at
startup and permit password login without an email confirmation service.
For this demo, the email address entered at registration is the Identity
username. Run it on loopback under the Development environment.

<demo:patch id="blazor-local-identity" after="blazor-scaffold">
--- a/examples/projects/blazor-todos/app/Program.cs
+++ b/examples/projects/blazor-todos/app/Program.cs
@@ -29,7 +29,7 @@
 
 builder.Services.AddIdentityCore<ApplicationUser>(options =>
     {
-        options.SignIn.RequireConfirmedAccount = true;
+        options.SignIn.RequireConfirmedAccount = !builder.Environment.IsDevelopment();
         options.Stores.SchemaVersion = IdentitySchemaVersions.Version3;
     })
     .AddEntityFrameworkStores<ApplicationDbContext>()
@@ -43,6 +43,11 @@
 // Configure the HTTP request pipeline.
 if (app.Environment.IsDevelopment())
 {
+    // The demo starts from an empty local database. Apply the captured Identity
+    // migration before any account endpoint can receive a request.
+    Directory.CreateDirectory(Path.Combine(app.Environment.ContentRootPath, ".local"));
+    await using var scope = app.Services.CreateAsyncScope();
+    await scope.ServiceProvider.GetRequiredService<ApplicationDbContext>().Database.MigrateAsync();
     app.UseMigrationsEndPoint();
 }
 else
--- a/examples/projects/blazor-todos/app/appsettings.json
+++ b/examples/projects/blazor-todos/app/appsettings.json
@@ -1,6 +1,6 @@
 {
   "ConnectionStrings": {
-    "DefaultConnection": "DataSource=Data/app.db;Cache=Shared"
+    "DefaultConnection": "Data Source=.local/todos.db"
   },
   "Logging": {
     "LogLevel": {
--- a/examples/projects/blazor-todos/app/HickoryTodos.csproj
+++ b/examples/projects/blazor-todos/app/HickoryTodos.csproj
@@ -9,10 +9,6 @@
   </PropertyGroup>
 
   <ItemGroup>
-    <None Update="Data\app.db" CopyToOutputDirectory="PreserveNewest" ExcludeFromSingleFile="true" />
-  </ItemGroup>
-
-  <ItemGroup>
     <PackageReference Include="Microsoft.AspNetCore.Diagnostics.EntityFrameworkCore" Version="10.0.12" />
     <PackageReference Include="Microsoft.AspNetCore.Identity.EntityFrameworkCore" Version="10.0.12" />
     <PackageReference Include="Microsoft.EntityFrameworkCore.Sqlite" Version="10.0.12" />
</demo:patch>

From the repository root, run:

```sh
cd examples/projects/blazor-todos/app
dotnet run --launch-profile http
```

Open the loopback URL printed by the app. The account pages are available at
`/Account/Register` and `/Account/Login`.
</demo:doc>
