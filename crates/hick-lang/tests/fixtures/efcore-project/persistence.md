# Persistence and the final migration

<hick:project entry="project.md" />

<hick:file id="model" after="packages" path="app/NoteDb.cs">
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
</hick:file>

The migration target and snapshot record the final schema as fixed strings.
They do not derive it from the current entity classes.

<hick:copy id="schema">
        builder.HasAnnotation("ProductVersion", "10.0.0");
        builder.Entity("Note", b =>
        {
            b.Property<int>("Id").ValueGeneratedOnAdd().HasColumnType("INTEGER");
            b.Property<string>("Text").IsRequired().HasColumnType("TEXT");
            b.HasKey("Id");
            b.ToTable("Notes");
        });
</hick:copy>

<hick:file id="migration" after="model" path="app/Migrations/Initial.cs">
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;
[DbContext(typeof(NoteDb))]
[Migration("202610120001_Initial")]
public sealed class Initial : Migration
{
    protected override void Up(MigrationBuilder migration)
    {
        migration.CreateTable("Notes", columns: table => new
        {
            Id = table.Column<int>(type: "INTEGER", nullable: false)
                .Annotation("Sqlite:Autoincrement", true),
            Text = table.Column<string>(type: "TEXT", nullable: false)
        }, constraints: table => table.PrimaryKey("PK_Notes", x => x.Id));
    }
    protected override void Down(MigrationBuilder migration) => migration.DropTable("Notes");
    protected override void BuildTargetModel(ModelBuilder builder)
    {
<hick:paste select="#schema"/>    }
}
</hick:file>

<hick:file id="snapshot" after="migration" path="app/Migrations/NoteDbSnapshot.cs">
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
[DbContext(typeof(NoteDb))]
public sealed class NoteDbSnapshot : ModelSnapshot
{
    protected override void BuildModel(ModelBuilder builder)
    {
<hick:paste select="#schema"/>    }
}
</hick:file>
