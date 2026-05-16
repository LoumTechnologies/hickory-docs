"""HickClient — subprocess-based wrapper around the hick-query-edit CLI.

All methods call the hick-query-edit binary and parse its JSON output.
Can be upgraded to PyO3 FFI later without changing the public API.
"""

import json
import os
import subprocess


class HickClient:
    """Client for querying and editing hick documents via the CLI."""

    def __init__(self):
        self.bin = os.environ.get("HICK_QUERY_EDIT_BIN", "hick-query-edit")
        self.workspace = os.environ.get("WORKSPACE", "/workspace")
        self.provenance_dir = os.path.join(self.workspace, ".hick", "provenance")

    def parse(self, hick_file):
        """Parse a .hick file and return its AST as a dict."""
        cmd = [self.bin, "parse", hick_file]
        return self._run(cmd)

    def query(self, selector, file=None):
        """Find structural elements in generated output.

        Args:
            selector: Query string (e.g. "fn greet", "struct Config", "class App")
            file: Path to the generated output file

        Returns:
            List of match dicts with element_kind, element_name, file,
            start_line, end_line, output_text, and origins.
        """
        if file is None:
            raise ValueError("file is required for query")
        cmd = [
            self.bin,
            "query",
            selector,
            "--file",
            file,
            "--provenance",
            self._provenance_path(file),
        ]
        return self._run(cmd)

    def structure(self, file):
        """Get structural summary of a generated file.

        Args:
            file: Path to the generated output file

        Returns:
            Multi-line string showing structural elements with line ranges.
        """
        cmd = [
            self.bin,
            "structure",
            file,
            "--provenance",
            self._provenance_path(file),
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, check=False)
        if result.returncode != 0:
            raise RuntimeError(f"hick-query-edit error: {result.stderr}")
        return result.stdout

    def provenance(self, file, start_line, end_line):
        """Get provenance info for a line range in generated output.

        Args:
            file: Path to the generated output file
            start_line: Start line (1-based)
            end_line: End line (1-based, inclusive)

        Returns:
            List of provenance span dicts with output_start, output_end, origin.
        """
        cmd = [
            self.bin,
            "provenance",
            file,
            "--start",
            str(start_line),
            "--end",
            str(end_line),
            "--provenance",
            self._provenance_path(file),
        ]
        return self._run(cmd)

    def edit(self, selector, new_content, file=None):
        """Compute edits to replace a selector match with new content.

        Args:
            selector: Selector for the element to replace (e.g. "fn greet")
            new_content: New source text to replace the matched element
            file: Path to the generated output file

        Returns:
            List of edit dicts with file, source_start, source_end,
            new_content, confidence.
        """
        if file is None:
            raise ValueError("file is required for edit")
        cmd = [
            self.bin,
            "compute-edits",
            file,
            "--selector",
            selector,
            "--new-content",
            new_content,
            "--provenance",
            self._provenance_path(file),
        ]
        return self._run(cmd)

    def apply(self, edits):
        """Apply computed edits to .hick source files.

        Args:
            edits: List of edit dicts (as returned by edit())
        """
        tmp_dir = os.path.join(self.workspace, ".hick", "tmp")
        os.makedirs(tmp_dir, exist_ok=True)
        edits_file = os.path.join(tmp_dir, "edits.json")

        with open(edits_file, "w") as f:
            json.dump(edits, f)

        cmd = [
            self.bin,
            "apply-edits",
            "--edits",
            edits_file,
            "--hick-dir",
            self.workspace,
        ]
        return self._run(cmd)

    def is_pipeline_owned(self, output_file):
        """Check whether a file is owned by the hick pipeline.

        Returns True if the file is declared in a <hick:file path="…"> element,
        False otherwise (in which case a direct write is appropriate).
        """
        try:
            owned = self.pipeline_list_owned()
            return any(f["output_path"] == output_file for f in (owned or []))
        except Exception:
            return False

    def pipeline_list_owned(self):
        """List all output files owned by the pipeline.

        Returns a list of dicts with output_path, hick_source, paste_slots.
        """
        cmd = [
            self.bin,
            "pipeline-list-owned",
            "--hick-dir",
            self.workspace,
        ]
        return self._run(cmd)

    def list_paste_slots(self, output_file):
        """List named paste slots available for a pipeline-owned output file.

        Args:
            output_file: Relative path of the generated output file.

        Returns:
            List of slot name strings.
        """
        cmd = [
            self.bin,
            "pipeline-list-slots",
            output_file,
            "--hick-dir",
            self.workspace,
        ]
        return self._run(cmd) or []

    def pipeline_write(self, output_file, content, slot=None):
        """Write to a pipeline-owned file by appending a hick:copy element.

        For pipeline-owned files this is the preferred way to make edits —
        it preserves provenance and is merge-safe across concurrent agents.
        Falls back to a direct write (with a printed warning) when the file
        is not owned by the pipeline.

        Args:
            output_file: Relative path of the generated output file to modify.
            content: New content to insert at the paste slot.
            slot: Named paste slot to target. Uses the first available slot if
                  omitted.

        Returns:
            Dict with element, hick_source, slot on success; None on fallback.
        """
        if not self.is_pipeline_owned(output_file):
            print(
                f"[hick] WARNING: '{output_file}' is not pipeline-owned — "
                "falling back to direct write",
                flush=True,
            )
            abs_path = os.path.join(self.workspace, output_file)
            os.makedirs(os.path.dirname(abs_path) or ".", exist_ok=True)
            with open(abs_path, "w") as fh:
                fh.write(content)
            return None

        cmd = [
            self.bin,
            "pipeline-write",
            output_file,
            "--content",
            content,
            "--hick-dir",
            self.workspace,
        ]
        if slot:
            cmd += ["--slot", slot]
        return self._run(cmd)

    def pipeline_new_file(self, output_file, content):
        """Create a new output file by appending a hick:file element to the pipeline.

        Args:
            output_file: Relative path for the new generated file.
            content: File content (may include <hick:paste name="…"/> slots).

        Returns:
            None on success; raises RuntimeError on failure.
        """
        cmd = [
            self.bin,
            "pipeline-new-file",
            output_file,
            "--content",
            content,
            "--hick-dir",
            self.workspace,
        ]
        return self._run(cmd)

    def _provenance_path(self, file):
        """Convention: provenance stored as .hick/provenance/<safe-name>.json"""
        if file:
            safe = file.replace("/", "__")
            return os.path.join(self.provenance_dir, f"{safe}.json")
        return os.path.join(self.provenance_dir, "all.json")

    def _run(self, cmd):
        """Execute a hick-query-edit command and parse JSON output."""
        result = subprocess.run(cmd, capture_output=True, text=True, check=False)
        if result.returncode != 0:
            raise RuntimeError(f"hick-query-edit error: {result.stderr}")
        if result.stdout.strip():
            return json.loads(result.stdout)
        return None
