namespace ExplicitWeb.Security;

public sealed class RateLimit
{
    private readonly object gate = new();
    private readonly Dictionary<string, (DateTimeOffset Start, int Count)> windows = new();
    private readonly TimeProvider clock;
    private readonly int limit;
    private readonly TimeSpan window;
    public RateLimit(TimeProvider clock, int limit, TimeSpan window)
    {
        if (limit < 1 || window <= TimeSpan.Zero) throw new ArgumentException("Use a positive rate limit and window.");
        (this.clock, this.limit, this.window) = (clock, limit, window);
    }
    public bool Allow(string key)
    {
        lock (gate)
        {
            var now = clock.GetUtcNow();
            foreach (var expired in windows.Where(x => now - x.Value.Start >= window).Select(x => x.Key).ToArray()) windows.Remove(expired);
            if (!windows.TryGetValue(key, out var current))
            {
                if (windows.Count >= 10_000) return false;
                current = (now, 0);
            }
            windows[key] = (current.Start, current.Count + 1);
            return current.Count < limit;
        }
    }
}
