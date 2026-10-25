<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/tests/fixtures/efcore-project/app/Migrations/NoteDbSnapshot.cs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1c6a2d9a117a0c6fd524c75a253b0f7052254546ae9aa04a9af582956880e377`.

<book:entry path="crates/hick-lang/tests/fixtures/efcore-project/app/Migrations/NoteDbSnapshot.cs">
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
</book:entry>

</book:doc>
