# Contract: docs/guarantees/basic-app.md
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tests"))
from support import HERE, build

CASES = [
    ("raw_html", 'Response Bad(string raw) => Response.Page(raw);', "CS1503", 'object Bad(string raw) => U.Page(raw);'),
    ("open_redirect", 'Response Bad(string raw) => Response.Redirect(raw);', "CS1503", 'object Bad(string raw) => U.Redirect(raw);'),
    ("unvalidated_title", 'TaskItem Bad(Tasks tasks, WritePermit permit, string raw) => tasks.Create(permit, raw);', "CS1503", 'object Bad(object tasks, object permit, string raw) => U.Create(permit, raw);'),
    ("missing_identity", 'object Bad(Tasks tasks, string id) => tasks.List(id);', "CS1503", 'object Bad(object tasks, string id) => U.List(id);'),
    ("missing_mutation_authority", 'object Bad(Tasks tasks, Principal user, Title title) => tasks.Create(user, title);', "CS1503", 'object Bad(object tasks, object user, string title) => U.Create(user, title);'),
    ("forged_permit", 'object Bad() => new WritePermit("victim", Guid.NewGuid());', "CS1729", 'object Bad() => new U.Permit("victim", Guid.NewGuid());'),
    ("missing_csrf_on_login", 'object Bad(Accounts accounts, Credentials credentials) => accounts.Login(credentials);', "CS7036", 'object Bad(object accounts, object credentials) => U.Login(credentials);'),
    ("wrong_route_parameter", 'object Bad(Route<Guid> route, string raw) => route.Link(raw);', "CS1503", 'object Bad(object route, string raw) => U.Link(raw);'),
    ("preformatted_sql", 'int Bad(Sql sql, string raw) => sql.Execute(raw);', "CS1503", 'int Bad(object sql, string raw) => U.Execute(raw);'),
    ("unchecked_transport_response", 'Task<Cors.Prepared> Bad() => Task.FromResult(Response.Text("unchecked"));', "CS0029", 'Task<object> Bad() => Task.FromResult(U.Page("unchecked"));'),
    ("forged_html", 'object Bad(string raw) => new Html(raw);', "CS1729", 'object Bad(string raw) => new U.Html(raw);'),
    ("unhandled_validation_result", 'object Bad(string raw) => Title.Parse(raw).Value;', "CS1061", 'object Bad(string raw) => U.Parse(raw).Value;'),
]
PREFIX = "using System; using System.Threading.Tasks; using ExplicitWeb; using ExplicitWeb.Data; using ExplicitWeb.Security; using TaskApp; class Witness { "
UNTYPED = """
static class U {
 public static object Page(string x)=>x; public static object Redirect(string x)=>x;
 public static object Create(object p,string x)=>x; public static object List(string x)=>x;
 public static object Login(object x)=>x; public static object Link(string x)=>x; public static int Execute(string x)=>0;
 public sealed record Permit(string User,Guid Request); public sealed record Html(string Raw);
 public sealed record Parsed(string Value); public static Parsed Parse(string x)=>new(x);
}
"""

def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--output", type=Path, required=True); args = parser.parse_args()
    build()
    records = []
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        references = ''.join('<Reference Include="'+name+'"><HintPath>'+str(HERE / 'Server/bin/Debug/net10.0' / (name+'.dll'))+'</HintPath></Reference>' for name in ['Core','Data','Security','Server'])
        (root / "Witness.csproj").write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><ImplicitUsings>enable</ImplicitUsings><TreatWarningsAsErrors>true</TreatWarningsAsErrors></PropertyGroup><ItemGroup>'+references+'</ItemGroup></Project>')
        source = root / "Witness.cs"
        def compile(body, extra=""):
            source.write_text(PREFIX + body + " }" + extra)
            start = time.perf_counter()
            result = subprocess.run(["dotnet", "build", str(root), "--nologo", "-v:q", "-m:1", "-nr:false"], cwd=HERE, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=90)
            return result, time.perf_counter() - start
        valid, _ = compile('Response Good(string raw) => Response.Page(Html.Template($"<p>{raw}</p>"));')
        assert valid.returncode == 0, valid.stdout
        for name, typed, diagnostic, untyped in CASES:
            result, elapsed = compile(typed)
            assert result.returncode != 0 and diagnostic in result.stdout, result.stdout
            control, control_elapsed = compile(untyped, UNTYPED)
            assert control.returncode == 0, control.stdout
            records.append(dict(name=name, expected_diagnostic=diagnostic, typed_rejected=True, untyped_compiled=True,
                typed_seconds=elapsed, untyped_seconds=control_elapsed, fixture_sha256=hashlib.sha256((typed+untyped).encode()).hexdigest()))
        escapes = []
        for name, code in [
            ("deliberate_null_suppression", 'Response Escape() => Response.Page(null!);'),
            ("incorrect_cors_policy", 'Cors Escape() => new Cors(new[]{new Uri("https://wrong.example")}, new[]{"GET"}, Array.Empty<string>());'),
        ]:
            result, elapsed = compile(code); assert result.returncode == 0, result.stdout
            escapes.append(dict(name=name, compiled=True, seconds=elapsed))
    sources={str(p.relative_to(HERE)):hashlib.sha256(p.read_bytes()).hexdigest() for p in HERE.rglob("*.cs") if "obj" not in p.parts and "bin" not in p.parts}
    report = dict(source_sha256=sources, sdk=subprocess.check_output(["dotnet","--version"],cwd=HERE,text=True).strip(), kind="selected_api_constraint_witnesses", machine=platform.platform(), python=platform.python_version(), cases=records,
        surviving_controls=escapes, limitation="Selected API-level fixtures versus an untyped control, not production bug frequency, a full competing framework, or AI productivity.")
    args.output.parent.mkdir(parents=True, exist_ok=True); args.output.write_text(json.dumps(report, indent=2)+"\n")
    print(f"PASS: {len(records)} selected mistakes rejected; {len(escapes)} deliberate escape/policy controls still compile")
if __name__ == "__main__": main()
