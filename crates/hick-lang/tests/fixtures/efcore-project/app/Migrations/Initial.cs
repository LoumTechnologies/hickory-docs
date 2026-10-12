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
