<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Measure specific claims without claiming general superiority

For researchers deciding whether this architecture deserves more investment. The compiler matrix enumerates intentionally selected mistakes and escape hatches. The host benchmark holds application behavior constant. Neither establishes AI productivity or a population-wide bug-prevention rate. An agent trial needs matched tasks, independently reviewed outcomes, actual transcripts and measured token counts.

## Compiler matrix

Compile one valid consumer first, then each selected mistaken consumer against the real assemblies. Require the expected diagnostic, not merely any build failure. Compile the corresponding untyped control separately. Keep deliberate null suppression and wrong CORS configuration as surviving controls, so the report cannot imply all web bugs are type errors.

<web:copy id="ew-compiler-matrix">
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
</web:copy>

## Host benchmark

Measure Release builds over real loopback HTTP. Seed both arms identically, warm them, alternate their order using a recorded seed, and retain per-request latencies, errors, sample sizes and source hashes. Persistent connections and fixed concurrency reduce connection setup noise. This is a local microbenchmark with shared business logic, not a claim that one web framework is faster overall.

<web:copy id="ew-host-benchmark">
import argparse
import concurrent.futures
import hashlib
import http.client
import json
from pathlib import Path
import platform
import random
import statistics
import sys
import tempfile
import time
import urllib.parse
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tests"))
from support import HERE, Client, command, server

def percentile(values, fraction):
    ordered = sorted(values); return ordered[min(len(ordered)-1, int((len(ordered)-1)*fraction))]

def sample(address, cookie, path, count, concurrency):
    url = urllib.parse.urlsplit(address)
    def worker(n):
        connection = http.client.HTTPConnection(url.hostname, url.port, timeout=10)
        latencies, errors = [], []
        try:
            for _ in range(n):
                start = time.perf_counter_ns()
                connection.request("GET", path, headers={"Cookie": cookie})
                response = connection.getresponse(); payload = response.read()
                latencies.append((time.perf_counter_ns()-start)/1e6)
                if response.status != 200: errors.append(dict(status=response.status, body=payload.decode(errors="replace")))
        finally: connection.close()
        return latencies, errors
    start = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(concurrency) as pool:
        results = list(pool.map(worker, [count//concurrency + (i < count%concurrency) for i in range(concurrency)]))
    elapsed = time.perf_counter()-start
    latencies = [x for group, _ in results for x in group]; errors = [x for _, group in results for x in group]
    return dict(requests=len(latencies), seconds=elapsed, requests_per_second=len(latencies)/elapsed,
        p50_ms=percentile(latencies,.5), p95_ms=percentile(latencies,.95), errors=errors, latencies_ms=latencies)

def main():
    parser=argparse.ArgumentParser(); parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--rounds",type=int,default=5); parser.add_argument("--requests",type=int,default=400)
    parser.add_argument("--concurrency",type=int,default=4); args=parser.parse_args()
    if min(args.rounds,args.requests,args.concurrency)<1: parser.error("Counts must be positive")
    for name in ["Server","Baseline"]: command("dotnet","build",f"{name}/{name}.csproj","-c","Release","--nologo")
    records=[]; rng=random.Random(173)
    with tempfile.TemporaryDirectory() as direct_dir, tempfile.TemporaryDirectory() as pipeline_dir:
        with server(direct_dir, "Server", configuration="Release") as direct, server(pipeline_dir,"Baseline",configuration="Release") as pipeline:
            clients={"direct":Client(direct),"pipeline":Client(pipeline)}
            for client in clients.values():
                client.register("benchmark")
                for i in range(25): assert client.write("POST","/api/tasks",{"title":f"Task {i}"})[0]==201
            for path in ["/health","/api/tasks"]:
                for client in clients.values(): sample(client.address,client.cookie,path,100,args.concurrency)
                for round in range(args.rounds):
                    arms=list(clients); rng.shuffle(arms)
                    for order, arm in enumerate(arms):
                        client=clients[arm]; result=sample(client.address,client.cookie,path,args.requests,args.concurrency)
                        records.append(dict(workload=path,round=round,order=order,arm=arm,**result))
                        assert not result["errors"], result["errors"]
    sources={str(p.relative_to(HERE)):hashlib.sha256(p.read_bytes()).hexdigest() for p in HERE.rglob("*.cs") if "obj" not in p.parts and "bin" not in p.parts}
    summary=[]
    for path in ["/health","/api/tasks"]:
        row={"workload":path}
        for arm in ["direct","pipeline"]:
            groups=[r for r in records if r["workload"]==path and r["arm"]==arm]
            row[arm]={"median_requests_per_second":statistics.median(r["requests_per_second"] for r in groups), "median_p95_ms":statistics.median(r["p95_ms"] for r in groups)}
        row["direct_over_pipeline_throughput"]=row["direct"]["median_requests_per_second"]/row["pipeline"]["median_requests_per_second"]
        summary.append(row)
    report=dict(kind="host_only_loopback_microbenchmark",machine=platform.platform(),python=platform.python_version(),
        sdk=command("dotnet","--version").stdout.strip(),configuration="Release",seed=173,concurrency=args.concurrency,
        rounds=args.rounds,requests_per_group=args.requests,source_sha256=sources,summary=summary,raw=records,
        limitation="Shared app and typed libraries in both arms; measures hosting only. Local load generator and server compete for resources. No AI productivity or general framework superiority claim.")
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps(summary,indent=2))
if __name__=="__main__": main()
</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/evaluation/compiler_matrix.py"><web:paste select="#ew-compiler-matrix"/></web:file>
<web:file path="experiments/explicit-web/evaluation/host_benchmark.py"><web:paste select="#ew-host-benchmark"/></web:file>

</web:doc>
