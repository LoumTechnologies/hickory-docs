# qwen3.8:27b: codeio-strip-type-arguments

Public and held-out checks passed in this reduced fixture, after 1 attempt(s).

The task requires implementing a helper method that strips generic type arguments from a C# type name string. The logic is straightforward: find the first occurrence of the `<` character. If it exists, return the substring before it; otherwise, return the original string. This approach preserves any whitespace and does not attempt to parse or validate the type structure, adhering to the contract.

```csharp
        public static string GetWithoutTypeArguments(string type)
        {
            if (string.IsNullOrEmpty(type))
                return type;

            int index = type.IndexOf('<');
            if (index == -1)
                return type;

            return type.Substring(0, index);
        }
```