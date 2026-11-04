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
