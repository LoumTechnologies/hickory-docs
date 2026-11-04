using System.Globalization;
using System.Net;
using System.Text;

namespace ExplicitWeb;

public sealed class Html
{
    public string Encoded { get; }
    private Html(string encoded) => Encoded = encoded;
    public static Html Text(string value) => new(WebUtility.HtmlEncode(value));
    public static Html Template(FormattableString template)
    {
        var arguments = template.GetArguments().Select(x => x is Html html ? html.Encoded :
            WebUtility.HtmlEncode(Convert.ToString(x, CultureInfo.InvariantCulture) ?? "")).Cast<object>().ToArray();
        return new Html(string.Format(CultureInfo.InvariantCulture, template.Format, arguments));
    }
    public static Html Join(IEnumerable<Html> fragments) => new(string.Concat(fragments.Select(x => x.Encoded)));
}

public sealed class LocalUrl
{
    public string Value { get; }
    private LocalUrl(string value) => Value = value;
    public static Result<LocalUrl> Parse(string? value)
    {
        if (string.IsNullOrEmpty(value) || value[0] != '/' || value.StartsWith("//", StringComparison.Ordinal) ||
            value.Any(c => char.IsControl(c) || c == '\\' || c == '%'))
            return Result<LocalUrl>.Fail(400, "Use an absolute local path without encoded characters.");
        return Result<LocalUrl>.Ok(new LocalUrl(value));
    }
}
