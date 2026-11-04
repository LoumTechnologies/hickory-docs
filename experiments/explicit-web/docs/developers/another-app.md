# Build another application

For C# developers reusing the framework rather than modifying the task app.

1. Create a net10.0 executable referencing Core, Data, Security and Hosting.
   Core has no dependency on ASP.NET. Hosting alone references the shared ASP.NET
   runtime. Your app does not need an HTTP context or a middleware container.
2. Construct Database with an absolute file path. Call Migrate with
   IdentitySchema.Migration followed by your own versioned schema. Append future
   migrations; do not change migrations used by an existing database.
3. Construct Sessions, Accounts, Cors and any repositories as concrete values.
   Resolve configuration once at startup and reject invalid values before binding.
4. Write `Task<Cors.Prepared> Handle(Request request)`. Inspect CORS first; return
   its Stop result when present. Load the session where your application needs it.
5. Define routes with Routes.At or Routes.Id. Match returns a checked route value;
   Link accepts that same parameter type. Parse body data with Form or Json, then
   validate domain values through a Result-returning constructor.
6. For protected reads, RequireUser returns Principal. For mutations, Verify the
   submitted CSRF token, then call Principal.Authorize with the VerifiedInput from
   that same session. Pass the resulting WritePermit and validated values to your
   repository. Include `permit.UserId` in every ownership predicate.
7. Render Html.Template with dynamic values in quoted text/attribute slots. A
   composed Html value is already encoded. Never interpolate into scripts, CSS,
   tag names or attribute names. Redirect requires LocalUrl.Parse. SQL Execute
   and Query accept FormattableString; interpolate values rather than formatting
   them to a string first. Keep table/column identifiers literal.
8. Return the response through the origin decision’s Apply method. Pass your
   handler to Server.Run. It implements IHttpApplication directly, bounds input,
   commits buffered bytes, logs exceptions privately, and handles shutdown.

Result.Match requires both a success and failure branch. That makes omissions
visible; it cannot prove that your failure branch is sensible. The compiler also
cannot check an arbitrary SQL ownership predicate. The sample repository’s
capability parameters make the required actor explicit, and its HTTP tests check
foreign-owned IDs. Add those tests for every new aggregate.

There are no global service locators or registration-order conventions. If a new
check is needed, call it in the request function and introduce a checked value
only where it makes an invalid call harder to express. Do not add a state type
for every incidental implementation line.
