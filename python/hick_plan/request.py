"""Operator request logging for missing operators."""

import json
import sys


def request_operator(name: str, description: str, example_usage: str = "") -> None:
    """Log a request for a missing operator to stderr (captured by host)."""
    payload = {
        "type": "operator_request",
        "name": name,
        "description": description,
        "example_usage": example_usage,
    }
    sys.stderr.write(json.dumps(payload) + "\n")
    sys.stderr.flush()
