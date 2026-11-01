# codeio-strip-type-arguments

Implement GetWithoutTypeArguments: return the prefix before the first < character, or the original string if none. Keep whitespace unchanged. This helper does not parse or validate C# types.

## Historical solution

Retained from [nloum/CodeIO at a5640ad8755b](https://github.com/nloum/CodeIO/blob/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631/FluentSourceGenerators/Utilities.cs#L149). This is a method projection with the [fixture limits](../../../../developers/historical-code-benchmark.md); the prompt is retrospective.

```csharp
        public static string GetWithoutTypeArguments(string type)
        {
            var idx = type.IndexOf('<');
            if (idx >= 0)
            {
                return type.Substring(0, idx);
            }

            return type;
        }
```

## Public acceptance example

```csharp
Equal(GetWithoutTypeArguments("Map<string, int>"),"Map"); Equal(GetWithoutTypeArguments("Int32"),"Int32");
```
