using System.ComponentModel.DataAnnotations;

namespace HickoryTodos.Models;

public sealed class TodoItem
{
    public int Id { get; set; }
    public string OwnerId { get; set; } = string.Empty;

    [Editable(true), Required, StringLength(120)]
    public string Title { get; set; } = string.Empty;

    [Editable(true)]
    public bool Completed { get; set; }
}
