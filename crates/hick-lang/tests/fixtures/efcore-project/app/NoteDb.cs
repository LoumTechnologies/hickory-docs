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
