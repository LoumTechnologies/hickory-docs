"""Tests for the hick_plan pipeline builder."""

import unittest

from hick_plan import Classification, Pipeline
from hick_plan.token import TokenAttenuator


class TestPipelineBuilder(unittest.TestCase):
    def test_simple_source_to_xml(self):
        xml = (
            Pipeline.source("employees", "csv", path="employees.csv")
            .column("id", "int32", key=True)
            .column("name", "utf8")
            .to_xml()
        )
        self.assertIn("<hick:pipeline", xml)
        self.assertIn('name="employees"', xml)
        self.assertIn('type="csv"', xml)
        self.assertIn('path="employees.csv"', xml)
        self.assertIn('name="id" type="int32" key="true"', xml)
        self.assertIn('name="name" type="utf8"', xml)
        self.assertIn("</hick:pipeline>", xml)

    def test_filter_wraps_source(self):
        xml = (
            Pipeline.source("data", "csv")
            .column("age", "int32")
            .filter("age > 18")
            .to_xml()
        )
        self.assertIn("<hick:where", xml)
        self.assertIn('expr="age &gt; 18"', xml)
        self.assertIn("<hick:data-source", xml)

    def test_mask_wraps_source(self):
        xml = (
            Pipeline.source("data", "csv")
            .column("ssn", "utf8")
            .mask([
                {"name": "ssn", "operation": "hash", "salt": "test-salt"},
            ])
            .to_xml()
        )
        self.assertIn("<hick:mask>", xml)
        self.assertIn('name="ssn" operation="hash"', xml)
        self.assertIn('salt="test-salt"', xml)

    def test_select_projects_columns(self):
        xml = (
            Pipeline.source("data", "csv")
            .column("id", "int32")
            .column("name", "utf8")
            .column("secret", "utf8")
            .select("id", "name")
            .to_xml()
        )
        self.assertIn("<hick:select", xml)
        self.assertIn("id,name", xml)

    def test_group_by(self):
        xml = (
            Pipeline.source("data", "csv")
            .column("dept", "utf8")
            .column("salary", "float64")
            .group_by("dept")
            .to_xml()
        )
        self.assertIn("<hick:group-by", xml)
        self.assertIn("dept", xml)

    def test_subscribe_with_slots(self):
        xml = (
            Pipeline.source("data", "csv")
            .column("recipient", "utf8")
            .column("body", "utf8")
            .subscribe("email", {"to": "recipient", "body": "body"})
            .to_xml()
        )
        self.assertIn('<hick:subscribe sink="email">', xml)
        self.assertIn('name="to" column="recipient"', xml)
        self.assertIn('name="body" column="body"', xml)

    def test_output_parquet(self):
        xml = (
            Pipeline.source("data", "csv")
            .column("id", "int32")
            .output("output.parquet", mode="materialized")
            .to_xml()
        )
        self.assertIn("<hick:parquet-file", xml)
        self.assertIn('path="output.parquet"', xml)
        self.assertIn('mode="materialized"', xml)

    def test_chained_pipeline(self):
        xml = (
            Pipeline.source("employees", "csv", path="emp.csv")
            .column("id", "int32", key=True)
            .column("ssn", "utf8", classification=Classification([{"sensitivity": "secret"}]))
            .column("name", "utf8")
            .column("salary", "float64")
            .mask([
                {"name": "ssn", "operation": "hash", "salt": "proj-salt"},
                {"name": "salary", "operation": "bucket", "bucket_size": 10000},
            ])
            .filter("salary > 50000")
            .select("id", "name", "ssn", "salary")
            .output("report.parquet")
            .to_xml()
        )
        # Check proper nesting order
        self.assertIn("<hick:parquet-file", xml)
        self.assertIn("<hick:select", xml)
        self.assertIn("<hick:where", xml)
        self.assertIn("<hick:mask>", xml)
        self.assertIn("<hick:data-source", xml)

    def test_classification_in_column(self):
        cls = Classification([{"sensitivity": "secret", "type": "ssn"}])
        xml = (
            Pipeline.source("data", "csv")
            .column("ssn", "utf8", classification=cls)
            .to_xml()
        )
        self.assertIn("classification=", xml)
        self.assertIn("secret", xml)

    def test_attribute_escaping(self):
        xml = (
            Pipeline.source("data & more", "csv")
            .column("col", "utf8")
            .filter('name == "O\'Brien" & age < 30')
            .to_xml()
        )
        self.assertIn("&amp;", xml)
        self.assertIn("&lt;", xml)


class TestTokenAttenuator(unittest.TestCase):
    def test_api_sink_caveat(self):
        caveats = (
            TokenAttenuator()
            .restrict_api_sink("email")
            .to_caveats()
        )
        self.assertEqual(caveats, ["api-sink = email"])

    def test_multiple_caveats(self):
        caveats = (
            TokenAttenuator()
            .restrict_api_sink("email")
            .set_expiry("2026-03-01T00:00:00Z")
            .set_max_calls("email", 1)
            .restrict_recipient("email", "alice@co.com")
            .to_caveats()
        )
        self.assertEqual(len(caveats), 4)
        self.assertIn("api-sink = email", caveats)
        self.assertIn("expires-at = 2026-03-01T00:00:00Z", caveats)
        self.assertIn("max-calls = email:1", caveats)
        self.assertIn("recipient = email:alice@co.com", caveats)


class TestClassification(unittest.TestCase):
    def test_json_roundtrip(self):
        cls = Classification([{"sensitivity": "secret"}])
        json_str = cls.to_json()
        restored = Classification.from_json(json_str)
        self.assertEqual(cls, restored)

    def test_union(self):
        a = Classification([{"sensitivity": "secret"}])
        b = Classification([{"trust": "internal"}])
        merged = a.union(b)
        self.assertEqual(len(merged.labels), 2)

    def test_union_deduplicates(self):
        a = Classification([{"sensitivity": "secret"}])
        b = Classification([{"sensitivity": "secret"}, {"trust": "internal"}])
        merged = a.union(b)
        self.assertEqual(len(merged.labels), 2)


if __name__ == "__main__":
    unittest.main()
