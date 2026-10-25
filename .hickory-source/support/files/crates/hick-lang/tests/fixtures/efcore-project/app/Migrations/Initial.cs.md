<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/tests/fixtures/efcore-project/app/Migrations/Initial.cs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b61b28d5c1c7b913ab4ad60c025baf4b8acf66089ec5e2abb6e80bba6e8eec19`.

<book:entry path="crates/hick-lang/tests/fixtures/efcore-project/app/Migrations/Initial.cs">
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
        builder.HasAnnotation("ProductVersion", "10.0.0");
        builder.Entity("Note", b =>
        {
            b.Property<int>("Id").ValueGeneratedOnAdd().HasColumnType("INTEGER");
            b.Property<string>("Text").IsRequired().HasColumnType("TEXT");
            b.HasKey("Id");
            b.ToTable("Notes");
        });
    }
}
</book:entry>

</book:doc>
