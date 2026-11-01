# qwen3.8:27b: codeio-variable-name

Public and held-out checks passed in this reduced fixture, after 1 attempt(s).

The `GenerateVariableName` method needs to transform a type name into a valid variable name. Based on the requirements:

1.  If `isInterface` is true, remove the first character (typically 'I').
2.  Remove the first `<` and everything after it (to strip generic type parameters).
3.  Apply the `Camelize` helper to the result.

Here is the implementation:

```csharp
public static string GenerateVariableName(string typeName, bool isInterface)
{
    string name = typeName;

    if (isInterface)
    {
        name = name.Substring(1);
    }

    int genericIndex = name.IndexOf('<');
    if (genericIndex != -1)
    {
        name = name.Substring(0, genericIndex);
    }

    return name.Camelize();
}
```