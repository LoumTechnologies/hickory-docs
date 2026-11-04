using System.Security.Cryptography;
using System.Text;

namespace ExplicitWeb;

public sealed class Assets
{
    private readonly Dictionary<string, (string Css, string Tag)> styles = new(StringComparer.Ordinal);
    public void AddCss(LocalUrl path, string css)
    {
        var tag = "\"" + Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(css))) + "\"";
        styles.Add(path.Value, (css, tag));
    }
    public Response? Find(Request request)
    {
        if (!styles.TryGetValue(request.Path, out var asset)) return null;
        if (request.Method is not ("GET" or "HEAD")) return Response.Text("Use GET to retrieve this stylesheet.", 405).WithHeader("Allow", "GET, HEAD");
        var response = request.Header("If-None-Match") == asset.Tag ? Response.Empty(304) : Response.Css(asset.Css);
        return response.WithHeader("ETag", asset.Tag).WithHeader("Cache-Control", "public, max-age=300");
    }
}
