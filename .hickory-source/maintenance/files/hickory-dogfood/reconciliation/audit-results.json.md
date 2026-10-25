<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/reconciliation/audit-results.json

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/reconciliation/audit-results.json">
{
  "date": "2026-10-25",
  "paragraph_ranges": 3715,
  "contracts": 326,
  "negative_controls": [
    {
      "challenge": "missing paragraph interval",
      "result": "rejected"
    },
    {
      "challenge": "changed paragraph hash",
      "result": "rejected"
    },
    {
      "challenge": "wrong requirement destination",
      "result": "rejected"
    }
  ],
  "validation_code_sha256": "083068970a27665211b8086ef6492566f6fc2180da33cb211e9a1f4aa383cd9e",
  "paragraph_ledger_sha256": "5c3afa3ef557540ec1605dceae39d3a7388ea25644a37dcc51cecd50680d2b02",
  "runtime_witnesses": 17,
  "scope": "Paragraph accounting and named runtime assertions, not semantic equivalence or whole-application acceptance."
}
</maint:entry>
</maint:doc>
