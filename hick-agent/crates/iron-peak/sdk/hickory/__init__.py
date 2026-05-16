"""Hickory SDK — Python interface to hick-query-edit CLI.

Provides structural query, edit, and provenance operations on hick
documents and their generated output files.

Usage:
    from hickory import hick

    matches = hick.query("fn greet", file="src/lib.rs")
    edits = hick.edit("fn greet", "fn greet_v2() {}", file="src/lib.rs")
    hick.apply(edits)
"""

from hickory.client import HickClient

hick = HickClient()

__all__ = ["hick", "HickClient"]
