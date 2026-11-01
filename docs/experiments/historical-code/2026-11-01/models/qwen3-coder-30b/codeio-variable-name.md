# qwen3-coder:30b: codeio-variable-name

Public and held-out checks passed in this reduced fixture, after 1 attempt(s).

Looking at this task, I need to implement the `GenerateVariableName` method that:

1. If `isInterface` is true, remove the first character
2. Remove the first `<` and everything after it when present
3. Apply the `Camelize` helper to the remaining text

Let me trace through the examples:
- `GenerateVariableName("IReadOnlyList<T>",true)` should return `"readOnlyList"`
  - Since `isInterface` is true, remove first character: `"ReadOnlyList<T>"`
  - Remove everything after `<`: `"ReadOnlyList"`
  - Apply `Camelize`: `"readOnlyList"`

- `GenerateVariableName("DataStore",false)` should return `"dataStore"`
  - Since `isInterface` is false, no removal
  - No `<` to remove
  - Apply `Camelize`: `"dataStore"`

```csharp
public static string GenerateVariableName(string typeName, bool isInterface)
{
    var result = typeName;
    
    // If isInterface is true, remove the first character
    if (isInterface && result.Length > 0)
    {
        result = result.Substring(1);
    }
    
    // Remove the first < and everything after it when present
    int indexOfLessThan = result.IndexOf('<');
    if (indexOfLessThan >= 0)
    {
        result = result.Substring(0, indexOfLessThan);
    }
    
    // Apply the Camelize helper
    return result.Camelize();
}
```