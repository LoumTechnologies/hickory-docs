# codeio-exception-factory

Refactor ThrowException into public static InvalidOperationException MakeException. It must return, rather than throw, the exception. Preserve optional CallerFilePath, CallerLineNumber and CallerMemberName attributes and append exactly " (file:line in member)" to the supplied message. The caller chooses whether to throw.

## Historical solution

Retained from [nloum/CodeIO at c850a1b023ea](https://github.com/nloum/CodeIO/blob/c850a1b023ead10e9cad72fe45984ead40d02801/src/FluentSourceGenerators/Utilities.cs#L15). This is a method projection with the [fixture limits](../../../../developers/historical-code-benchmark.md); the prompt is retrospective.

```csharp
        public static InvalidOperationException MakeException(string errorMessage, [CallerFilePath] string callerFilePath = null,
            [CallerLineNumber] int callerLineNumber = 0, [CallerMemberName] string callerMemberName = null)
        {
            return new InvalidOperationException(errorMessage +
                                                $" ({callerFilePath}:{callerLineNumber} in {callerMemberName})");
        }
```

## Public acceptance example

```csharp
var e=MakeException("bad", "sample.cs", 17, "Build"); Equal(e.Message,"bad (sample.cs:17 in Build)");
```
