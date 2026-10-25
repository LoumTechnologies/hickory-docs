<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/tests/fixtures/efcore-project/app/NoteDb.cs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `995839958093d2de1afb0bd1dcd8ce9ee87422f9ed82fb644433a431985159e8`.

<book:entry path="crates/hick-lang/tests/fixtures/efcore-project/app/NoteDb.cs">
using Microsoft.EntityFrameworkCore;
public sealed class Note
{
    public int Id { get; set; }
    public string Text { get; set; } = "";
}
public sealed class NoteDb : DbContext
{
    public DbSet<Note> Notes => Set<Note>();
    protected override void OnConfiguring(DbContextOptionsBuilder options)
        => options.UseSqlite("Data Source=:memory:");
    protected override void OnModelCreating(ModelBuilder builder)
    {
        builder.Entity<Note>(b =>
        {
            b.ToTable("Notes");
            b.Property(n => n.Id).ValueGeneratedOnAdd().HasColumnType("INTEGER");
            b.Property(n => n.Text).IsRequired().HasColumnType("TEXT");
            b.HasKey(n => n.Id);
        });
    }
}
</book:entry>

</book:doc>
