using Microsoft.EntityFrameworkCore;
using var db = new NoteDb();
db.Database.OpenConnection();
db.Database.Migrate();
var note = new Note { Text = "verified" };
db.Notes.Add(note);
db.SaveChanges();
Console.WriteLine(db.Notes.Single().Text);
Console.WriteLine(db.Database.GetAppliedMigrations().Single());
