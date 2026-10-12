using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
[DbContext(typeof(NoteDb))]
public sealed class NoteDbSnapshot : ModelSnapshot
{
    protected override void BuildModel(ModelBuilder builder)
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
