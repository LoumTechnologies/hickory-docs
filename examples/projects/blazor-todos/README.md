# Blazor todos

This demo starts with the .NET 10 Blazor Server template and ASP.NET Core
Identity, adds an EF Core model, and generates the todo page from Hickory's
C# code model.

[Read the literate construction](../../../.hickory-source/demos/blazor-todos/scaffold.md).

Start the app from this directory.

```sh
cd app
dotnet run --launch-profile http
```

Open the loopback URL printed by the app. Register with an email address and
password, then choose **My todos**. The Development profile creates the SQLite
database and applies migrations on startup.
