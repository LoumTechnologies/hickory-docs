# codeio-variable-name

Implement GenerateVariableName. If isInterface is true remove the first character. Remove the first < and everything after it when present. Apply the supplied Camelize helper to the remaining text. Assume interface names are nonempty; preserve the behavior of the supplied helpers.

## Historical solution

Retained from [nloum/CodeIO at a5640ad8755b](https://github.com/nloum/CodeIO/blob/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631/FluentSourceGenerators/Utilities.cs#L199). This is a method projection with the [fixture limits](../../../../developers/historical-code-benchmark.md); the prompt is retrospective.

```csharp
        public static string GenerateVariableName(string typeName, bool isInterface)
        {
            var result = isInterface ? typeName.Substring(1) : typeName;
            if (result.Contains("<"))
            {
                result = result.Substring(0, result.IndexOf('<'));
            }

            return result.Camelize();
        }
```

## Public acceptance example

```csharp
Equal(GenerateVariableName("IReadOnlyList<T>",true),"readOnlyList"); Equal(GenerateVariableName("DataStore",false),"dataStore");
```
