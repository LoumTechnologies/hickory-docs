"""Focused real runner-listing revision acceptance in a disposable Rust workspace."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-runner-listing-") as temporary:
        root=Path(temporary)/"workspace";root.mkdir();(root/"src").mkdir()
        (root/"Cargo.toml").write_text('[package]\nname="listing_fixture"\nversion="0.1.0"\nedition="2024"\n')
        source='#[test] fn listed_case() { std::fs::write("executed-test-body", "ran").unwrap(); }\n'
        (root/"src/lib.rs").write_text(source)
        engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"runner-listing"],env={**os.environ,"HICKORY_STATE_DIR":str(Path(temporary)/"state")})
        try:
            origin=fixture.origin(engine)
            def discover():
                request=urllib.request.Request(origin+"/api/tests/coverage",data=json.dumps({"action":"discover","language":"rust","path":"."}).encode(),headers={"Content-Type":"application/json"})
                with urllib.request.urlopen(request,timeout=40) as response:return json.load(response)
            assert not (root/"Cargo.lock").exists()
            first=discover()
            assert first["version"]==1 and first["discovery"]=="runner",first
            assert "listed_case" in [test["id"] for test in first["tests"]],first
            assert first["source_hashes"]["src/lib.rs"]==hashlib.sha256(source.encode()).hexdigest()
            assert "Cargo.lock" not in first["source_hashes"],first
            assert first["source_changed_during_discovery"] is True and first["source_state"]=="historical",first
            assert first["source_frozen"] is False
            assert (root/"Cargo.lock").exists() and not (root/"executed-test-body").exists()
            second=discover()
            assert second["source_changed_during_discovery"] is False and second["source_state"]=="current",second
            assert "Cargo.lock" in second["source_hashes"] and second["fingerprint"]!=first["fingerprint"]
            (root/"src/lib.rs").write_text(source+"// later source\n")
            third=discover()
            assert third["source_hashes"]["src/lib.rs"]!=second["source_hashes"]["src/lib.rs"] and third["fingerprint"]!=second["fingerprint"]
            assert not (root/"executed-test-body").exists()
            print("PASS runner discovery: real Cargo names, pre-run revisions, lockfile mutation retired, later stable listing, unchanged test bodies")
        finally:fixture.stop(engine)
if __name__=="__main__":main()
