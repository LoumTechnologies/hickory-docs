"""Internal operator node classes that build XML fragments."""

import html
from typing import Any


def _escape(s: str) -> str:
    """Escape a string for use in XML attribute values."""
    return html.escape(s, quote=True)


def _indent(level: int) -> str:
    return "  " * level


class _ColumnDef:
    """A column definition for a data source."""

    def __init__(
        self,
        name: str,
        dtype: str,
        key: bool = False,
        classification: Any | None = None,
    ):
        self.name = name
        self.dtype = dtype
        self.key = key
        self.classification = classification

    def to_xml(self, indent: int) -> str:
        attrs = f'name="{_escape(self.name)}" type="{_escape(self.dtype)}"'
        if self.key:
            attrs += ' key="true"'
        if self.classification is not None:
            cls_json = self.classification.to_json() if hasattr(self.classification, "to_json") else str(self.classification)
            attrs += f" classification='{_escape(cls_json)}'"
        return f"{_indent(indent)}<hick:column {attrs}/>"


class _DataSourceNode:
    """A data source node."""

    def __init__(self, name: str, source_type: str, path: str | None = None):
        self.name = name
        self.source_type = source_type
        self.path = path
        self.columns: list[_ColumnDef] = []

    def to_xml(self, indent: int = 1) -> str:
        attrs = f'name="{_escape(self.name)}" type="{_escape(self.source_type)}"'
        if self.path:
            attrs += f' path="{_escape(self.path)}"'
        lines = [f"{_indent(indent)}<hick:data-source {attrs}>"]
        for col in self.columns:
            lines.append(col.to_xml(indent + 1))
        lines.append(f"{_indent(indent)}</hick:data-source>")
        return "\n".join(lines)


class _WhereNode:
    """A filter node."""

    def __init__(self, source: Any, expr: str):
        self.source = source
        self.expr = expr

    def to_xml(self, indent: int = 1) -> str:
        lines = [f'{_indent(indent)}<hick:where expr="{_escape(self.expr)}">']
        lines.append(self.source.to_xml(indent + 1))
        lines.append(f"{_indent(indent)}</hick:where>")
        return "\n".join(lines)


class _MaskColumnSpec:
    """A mask specification for a single column."""

    def __init__(
        self,
        name: str,
        operation: str,
        target_classification: Any | None = None,
        mask_value: str | None = None,
        bucket_size: float | None = None,
        salt: str | None = None,
    ):
        self.name = name
        self.operation = operation
        self.target_classification = target_classification
        self.mask_value = mask_value
        self.bucket_size = bucket_size
        self.salt = salt

    def to_xml(self, indent: int) -> str:
        attrs = f'name="{_escape(self.name)}" operation="{_escape(self.operation)}"'
        if self.target_classification is not None:
            cls_json = self.target_classification.to_json() if hasattr(self.target_classification, "to_json") else str(self.target_classification)
            attrs += f" target-classification='{_escape(cls_json)}'"
        if self.mask_value is not None:
            attrs += f' mask-value="{_escape(self.mask_value)}"'
        if self.bucket_size is not None:
            attrs += f' bucket-size="{self.bucket_size}"'
        if self.salt is not None:
            attrs += f' salt="{_escape(self.salt)}"'
        return f"{_indent(indent)}<hick:column {attrs}/>"


class _MaskNode:
    """A mask node."""

    def __init__(self, source: Any, specs: list[_MaskColumnSpec]):
        self.source = source
        self.specs = specs

    def to_xml(self, indent: int = 1) -> str:
        lines = [f"{_indent(indent)}<hick:mask>"]
        for spec in self.specs:
            lines.append(spec.to_xml(indent + 1))
        lines.append(self.source.to_xml(indent + 1))
        lines.append(f"{_indent(indent)}</hick:mask>")
        return "\n".join(lines)


class _SelectNode:
    """A select (projection) node."""

    def __init__(self, source: Any, columns: list[str]):
        self.source = source
        self.columns = columns

    def to_xml(self, indent: int = 1) -> str:
        cols = " ".join(f'"{_escape(c)}"' for c in self.columns)
        lines = [f'{_indent(indent)}<hick:select columns="{_escape(",".join(self.columns))}">']
        lines.append(self.source.to_xml(indent + 1))
        lines.append(f"{_indent(indent)}</hick:select>")
        return "\n".join(lines)


class _GroupByNode:
    """A group-by aggregation node."""

    def __init__(self, source: Any, columns: list[str]):
        self.source = source
        self.columns = columns

    def to_xml(self, indent: int = 1) -> str:
        lines = [f'{_indent(indent)}<hick:group-by columns="{_escape(",".join(self.columns))}">']
        lines.append(self.source.to_xml(indent + 1))
        lines.append(f"{_indent(indent)}</hick:group-by>")
        return "\n".join(lines)


class _SlotBinding:
    """A slot binding for subscribe."""

    def __init__(self, name: str, column: str):
        self.name = name
        self.column = column

    def to_xml(self, indent: int) -> str:
        return f'{_indent(indent)}<hick:slot name="{_escape(self.name)}" column="{_escape(self.column)}"/>'


class _SubscribeNode:
    """A subscribe (API sink dispatch) node."""

    def __init__(self, source: Any, sink_type: str, slots: list[_SlotBinding]):
        self.source = source
        self.sink_type = sink_type
        self.slots = slots

    def to_xml(self, indent: int = 1) -> str:
        lines = [f'{_indent(indent)}<hick:subscribe sink="{_escape(self.sink_type)}">']
        for slot in self.slots:
            lines.append(slot.to_xml(indent + 1))
        lines.append(self.source.to_xml(indent + 1))
        lines.append(f"{_indent(indent)}</hick:subscribe>")
        return "\n".join(lines)


class _OutputNode:
    """An output node (parquet file, stdout, etc.)."""

    def __init__(self, source: Any, path: str, mode: str = "materialized"):
        self.source = source
        self.path = path
        self.mode = mode

    def to_xml(self, indent: int = 1) -> str:
        lines = [
            f'{_indent(indent)}<hick:parquet-file path="{_escape(self.path)}" mode="{_escape(self.mode)}">'
        ]
        lines.append(self.source.to_xml(indent + 1))
        lines.append(f"{_indent(indent)}</hick:parquet-file>")
        return "\n".join(lines)
