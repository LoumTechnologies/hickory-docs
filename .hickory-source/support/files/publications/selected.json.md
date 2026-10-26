<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# publications/selected.json

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `11f1506b0d701d3eba846c07ddb9cd18d2d8e9b7f2feb28355a69ce9d777c555`.

<book:entry path="publications/selected.json">
{
  "version": 1,
  "documents": [
    {
      "path": "examples/ai-engineer-deadline.md",
      "reviewed_output": true,
      "dependencies": [
        "Cargo.lock",
        "rust-toolchain.toml"
      ],
      "assets": [],
      "requirements": [
        {
          "id": "R1",
          "text": "A request consumes at most its total budget across all attempts.",
          "witnesses": [
            0
          ]
        }
      ],
      "reviewed_inputs": {
        "examples/ai-engineer-deadline.md": "636ddbd13794d41faa0907e64e96bcf6d213a466f2dfd44423ae2b17a767099a",
        "Cargo.lock": "2281cc1289f143fb9ff06f6d113031e28276c3d59102e298210b0fd1e6ac7e50",
        "rust-toolchain.toml": "4d0674575b116a100c7d0ad754308dc6dea178b4808c760afc43136db967c7a6"
      }
    },
    {
      "path": "examples/security-authorization/vulnerable.md",
      "reviewed_output": true,
      "prediction_history": "examples/security-authorization/predictions.json",
      "expected_status": "failed",
      "expected_status_reason": "The synthetic existence-only authorization predicate must fail R-AUTH-1 before its tenant-ownership patch.",
      "requirements": [
        {
          "id": "R-AUTH-1",
          "text": "A tenant cannot access another tenant\u2019s record.",
          "witnesses": []
        },
        {
          "id": "R-LEGIT-1",
          "text": "Own-record access succeeds and missing records remain denied on both revisions.",
          "witnesses": []
        }
      ],
      "dependencies": [],
      "assets": [],
      "reviewed_inputs": {
        "examples/security-authorization/vulnerable.md": "20a184766b132a6f783e162d42546b70986c5d4c46807bd39d37a5728f87ffd9",
        "examples/security-authorization/predictions.json": "3111069cf15a8a14c7d5748bf4542415752783487b9003953d4f7d0f547cf94b"
      }
    },
    {
      "path": "examples/security-authorization/patched.md",
      "reviewed_output": true,
      "prediction_history": "examples/security-authorization/predictions.json",
      "expected_status": "passed",
      "expected_status_reason": "",
      "requirements": [
        {
          "id": "R-AUTH-1",
          "text": "A tenant cannot access another tenant\u2019s record.",
          "witnesses": []
        },
        {
          "id": "R-LEGIT-1",
          "text": "Own-record access succeeds and missing records remain denied on both revisions.",
          "witnesses": []
        }
      ],
      "dependencies": [],
      "assets": [],
      "reviewed_inputs": {
        "examples/security-authorization/patched.md": "b60f7ed89deae86c9026c7dfe1535e68f3f39d4cf8fb37fd186876f33db12102",
        "examples/security-authorization/predictions.json": "3111069cf15a8a14c7d5748bf4542415752783487b9003953d4f7d0f547cf94b"
      }
    }
  ]
}
</book:entry>


## Reviewed lockfile update, 2026-10-26

The Cargo.lock diff since the previously reviewed revision
`6ed67e75ed6e5b89b2967b673760bd35ceb257ab` was inspected before changing this pin.
It adds the coverage, plugin and profile workspace dependencies and source-map
registry dependencies with their lock metadata. No selected example source,
asset approval, requirement or expected status changed. This is a public-content
review of the dependency input; executable outcomes still require the selected
verification commands. No automatic review-hash refresh was introduced.

</book:doc>
