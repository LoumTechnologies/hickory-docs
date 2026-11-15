"""Run the completion protocol against an isolated real engine."""
import importlib.util
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("provider_fixture",ROOT/"scripts/test-provider-actions.py")
fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
if __name__=="__main__":fixture.run_provider_witness("provider-completions-driver.mjs",())
