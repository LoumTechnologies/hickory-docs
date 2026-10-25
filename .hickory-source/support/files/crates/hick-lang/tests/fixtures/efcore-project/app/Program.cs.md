<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Compiled Language Launches What A Build Produced](../../../../../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md)
- [An Output Path Declared Twice Warns Before It Runs](../../../../../../../../guarantees/execution/an-output-path-declared-twice-warns-before-it-runs/contract.md)
- [A Woven Fenced Block Has No Leading BOM](../../../../../../../../guarantees/verification/a-woven-fenced-block-has-no-leading-bom/contract.md)

Captured SHA-256: `edf6656f78179eadaf95d53ffbd270d90f5309076113caeb1a5b9f72b979b9be`.

<book:entry path="crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs">
using Microsoft.EntityFrameworkCore;
using var db = new NoteDb();
db.Database.OpenConnection();
db.Database.Migrate();
var note = new Note { Text = "verified" };
db.Notes.Add(note);
db.SaveChanges();
Console.WriteLine(db.Notes.Single().Text);
Console.WriteLine(db.Database.GetAppliedMigrations().Single());
</book:entry>

</book:doc>
