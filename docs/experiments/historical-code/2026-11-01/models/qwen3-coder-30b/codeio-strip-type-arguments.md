# qwen3-coder:30b: codeio-strip-type-arguments

Public and held-out checks passed in this reduced fixture, after 1 attempt(s).

Looking at this task, I need to implement a method that returns the prefix before the first `<` character in a string, or the original string if there's no `<` character.

Let me analyze the requirements:
1. Find the first `<` character in the string
2. Return everything before it
3. If no `<` exists, return the original string
4. Preserve all whitespace unchanged

From the example:
- `GetWithoutTypeArguments("Map<string, int>")` should return `"Map"`
- `GetWithoutTypeArguments("Int32")` should return `"Int32"`

This is a straightforward string operation using `IndexOf` and `Substring`.

```csharp
public static string GetWithoutTypeArguments(string type)
{
    int index = type.IndexOf('<');
    if (index == -1)
    {
        return type;
    }
    return type.Substring(0, index);
}
```