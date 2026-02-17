"""Pipeline builder for hick XML execution plans."""

import json
import sys
from typing import Any

from .classification import Classification
from .operators import (
    _ColumnDef,
    _DataSourceNode,
    _GroupByNode,
    _MaskColumnSpec,
    _MaskNode,
    _OutputNode,
    _SelectNode,
    _SlotBinding,
    _SubscribeNode,
    _WhereNode,
)
from .request import request_operator


class Pipeline:
    """Entry point for building a hick execution plan."""

    @staticmethod
    def source(
        name: str,
        source_type: str = "csv",
        path: str | None = None,
    ) -> "DataSourceBuilder":
        return DataSourceBuilder(name, source_type, path)


class DataSourceBuilder:
    """Builder for a data source with columns."""

    def __init__(self, name: str, source_type: str, path: str | None):
        self._node = _DataSourceNode(name, source_type, path)

    def column(
        self,
        name: str,
        dtype: str = "utf8",
        key: bool = False,
        classification: Classification | None = None,
    ) -> "DataSourceBuilder":
        self._node.columns.append(_ColumnDef(name, dtype, key, classification))
        return self

    def filter(self, expr: str) -> "PipelineStage":
        return PipelineStage(_WhereNode(self._node, expr))

    def mask(self, specs: list[dict[str, Any]]) -> "PipelineStage":
        mask_specs = []
        for s in specs:
            mask_specs.append(
                _MaskColumnSpec(
                    name=s["name"],
                    operation=s["operation"],
                    target_classification=s.get("target_classification"),
                    mask_value=s.get("mask_value"),
                    bucket_size=s.get("bucket_size"),
                    salt=s.get("salt"),
                )
            )
        return PipelineStage(_MaskNode(self._node, mask_specs))

    def select(self, *columns: str) -> "PipelineStage":
        return PipelineStage(_SelectNode(self._node, list(columns)))

    def group_by(self, *columns: str) -> "PipelineStage":
        return PipelineStage(_GroupByNode(self._node, list(columns)))

    def subscribe(
        self, sink_type: str, slots: dict[str, str]
    ) -> "PipelineStage":
        bindings = [_SlotBinding(name=k, column=v) for k, v in slots.items()]
        return PipelineStage(_SubscribeNode(self._node, sink_type, bindings))

    def output(self, path: str, mode: str = "materialized") -> "PipelineStage":
        return PipelineStage(_OutputNode(self._node, path, mode))

    def to_xml(self) -> str:
        return _wrap_xml(self._node.to_xml(1))


class PipelineStage:
    """An intermediate or terminal pipeline stage."""

    def __init__(self, node: Any):
        self._node = node

    def filter(self, expr: str) -> "PipelineStage":
        return PipelineStage(_WhereNode(self._node, expr))

    def mask(self, specs: list[dict[str, Any]]) -> "PipelineStage":
        mask_specs = []
        for s in specs:
            mask_specs.append(
                _MaskColumnSpec(
                    name=s["name"],
                    operation=s["operation"],
                    target_classification=s.get("target_classification"),
                    mask_value=s.get("mask_value"),
                    bucket_size=s.get("bucket_size"),
                    salt=s.get("salt"),
                )
            )
        return PipelineStage(_MaskNode(self._node, mask_specs))

    def select(self, *columns: str) -> "PipelineStage":
        return PipelineStage(_SelectNode(self._node, list(columns)))

    def group_by(self, *columns: str) -> "PipelineStage":
        return PipelineStage(_GroupByNode(self._node, list(columns)))

    def subscribe(
        self, sink_type: str, slots: dict[str, str]
    ) -> "PipelineStage":
        bindings = [_SlotBinding(name=k, column=v) for k, v in slots.items()]
        return PipelineStage(_SubscribeNode(self._node, sink_type, bindings))

    def output(self, path: str, mode: str = "materialized") -> "PipelineStage":
        return PipelineStage(_OutputNode(self._node, path, mode))

    def to_xml(self) -> str:
        return _wrap_xml(self._node.to_xml(1))

    def submit(self) -> None:
        """Write the plan XML to stdout."""
        sys.stdout.write(self.to_xml())
        sys.stdout.write("\n")
        sys.stdout.flush()


def _wrap_xml(body: str) -> str:
    return (
        '<hick:pipeline xmlns:hick="https://hickory.dev/hick">\n'
        + body
        + "\n</hick:pipeline>"
    )
