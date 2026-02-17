"""Classification helpers for data labeling."""

import json
from typing import Any


class Classification:
    """A set of JSON classification labels for a data column."""

    def __init__(self, labels: list[dict[str, Any]] | None = None):
        self.labels = labels or []

    def to_json(self) -> str:
        return json.dumps(self.labels)

    @classmethod
    def from_json(cls, s: str) -> "Classification":
        return cls(json.loads(s))

    def union(self, other: "Classification") -> "Classification":
        combined = list(self.labels)
        for label in other.labels:
            if label not in combined:
                combined.append(label)
        return Classification(combined)

    def __repr__(self) -> str:
        return f"Classification({self.labels!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Classification):
            return NotImplemented
        return self.labels == other.labels


def col(name: str) -> str:
    """Helper to reference a column name in expressions."""
    return name
