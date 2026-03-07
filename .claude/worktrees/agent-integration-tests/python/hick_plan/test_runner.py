"""Subprocess wrapper for hick-plan-runner."""

import json
import subprocess
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


@dataclass
class TestResult:
    """Result of running a plan through hick-plan-runner."""

    exit_code: int
    stdout: str
    stderr: str
    api_calls: list[dict[str, Any]] = field(default_factory=list)

    @property
    def success(self) -> bool:
        return self.exit_code == 0


def run_plan(
    xml: str,
    data_path: str | None = None,
    token_caveats: list[str] | None = None,
    mock: bool = True,
    runner_path: str = "hick-plan-runner",
    timeout: float = 30.0,
) -> TestResult:
    """Run a plan through hick-plan-runner and return the result.

    Args:
        xml: The hick XML plan string.
        data_path: Path to data files directory.
        token_caveats: List of token caveat strings.
        mock: If True, use mock mode for API calls.
        runner_path: Path to the hick-plan-runner binary.
        timeout: Timeout in seconds.
    """
    with tempfile.NamedTemporaryFile(mode="w", suffix=".xml", delete=False) as plan_file:
        plan_file.write(xml)
        plan_path = plan_file.name

    api_log_path = tempfile.mktemp(suffix=".json")

    cmd = [runner_path, "--plan", plan_path, "--api-log", api_log_path]

    if data_path:
        cmd.extend(["--data", data_path])

    if mock:
        cmd.extend(["--mode", "mock"])

    if token_caveats:
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as token_file:
            json.dump(token_caveats, token_file)
            cmd.extend(["--token", token_file.name])

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except FileNotFoundError:
        return TestResult(
            exit_code=127,
            stdout="",
            stderr=f"hick-plan-runner not found at: {runner_path}",
        )
    except subprocess.TimeoutExpired:
        return TestResult(
            exit_code=124,
            stdout="",
            stderr=f"hick-plan-runner timed out after {timeout}s",
        )

    api_calls = []
    try:
        api_calls = json.loads(Path(api_log_path).read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        pass

    return TestResult(
        exit_code=result.returncode,
        stdout=result.stdout,
        stderr=result.stderr,
        api_calls=api_calls,
    )
