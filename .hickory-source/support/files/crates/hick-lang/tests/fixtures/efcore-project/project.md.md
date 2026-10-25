<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/tests/fixtures/efcore-project/project.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `4a9e321188c6f3a31fec7e7935cbf1ed8ae751c2dba722becf798a6733a9fae6`.

<book:entry path="crates/hick-lang/tests/fixtures/efcore-project/project.md">
# Persist one note using the completed project

<hick:project sources="persistence.md" exclude=".git/ .hg/ .hick-cache/ app/bin/ app/obj/" />

The SDK selection is source configuration too, so a fresh Markdown-only copy
keeps it instead of relying on this repository's or this machine's defaults.

<hick:file id="sdk" path="global.json">
{"sdk":{"version":"10.0.401","rollForward":"latestPatch"}}
</hick:file>

<hick:file id="manifest" path="app/app.csproj">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
</Project>
</hick:file>

<hick:file id="program" path="app/Program.cs">
Console.WriteLine("initial");
</hick:file>

The caller first describes the persistence behavior. This local state has
neither the package nor the context yet; default New waits for the whole scope.

<hick:patch id="caller" after="manifest program">
--- a/app/Program.cs
+++ b/app/Program.cs
@@ -1 +1,9 @@
-Console.WriteLine("initial");
+using Microsoft.EntityFrameworkCore;
+using var db = new NoteDb();
+db.Database.OpenConnection();
+db.Database.Migrate();
+var note = new Note { Text = "verified" };
+db.Notes.Add(note);
+db.SaveChanges();
+Console.WriteLine(db.Notes.Single().Text);
+Console.WriteLine(db.Database.GetAppliedMigrations().Single());
</hick:patch>

<hick:patch id="packages" after="caller">
--- a/app/app.csproj
+++ b/app/app.csproj
@@ -5,4 +5,8 @@
     <ImplicitUsings>enable</ImplicitUsings>
     <Nullable>enable</Nullable>
   </PropertyGroup>
+  <ItemGroup>
+    <PackageReference Include="Microsoft.EntityFrameworkCore.Sqlite" Version="10.0.0" />
+    <PackageReference Include="SQLitePCLRaw.bundle_e_sqlite3" Version="3.0.5" />
+  </ItemGroup>
 </Project>
</hick:patch>
</book:entry>

</book:doc>
