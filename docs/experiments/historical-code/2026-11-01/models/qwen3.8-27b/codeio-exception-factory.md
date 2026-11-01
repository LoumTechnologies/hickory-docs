# qwen3.8:27b: codeio-exception-factory

Public and held-out checks passed in this reduced fixture, after 1 attempt(s).

The task is to refactor `ThrowException` into a new method `MakeException` that returns an `InvalidOperationException` instead of throwing it. The method must preserve the optional caller information attributes and append the formatted caller details to the message.

Here is the complete replacement method:

```csharp
public static InvalidOperationException MakeException(string errorMessage, [CallerFilePath] string callerFilePath = null,
    [CallerLineNumber] int callerLineNumber = 0, [CallerMemberName] string callerMemberName = null)
{
    return new InvalidOperationException(errorMessage +
        $" ({callerFilePath}:{callerLineNumber} in {callerMemberName})");
}
```