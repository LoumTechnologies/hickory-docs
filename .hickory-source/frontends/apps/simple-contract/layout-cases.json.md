<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Run the same pane scenarios in both languages

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

This fixture is the shared behavioral input, not a UI snapshot. Both implementations run each sequence and compare ordered tab groups. Index-based group selection lets platform implementations choose internal node identities freely.

<ui:copy id="simple-232e6fe962-0">
{
  "cases": [
    {
      "name": "split and move preserve one tab identity",
      "steps": [
        {
          "op": "open",
          "group": 1,
          "tab": "notes.md"
        },
        {
          "op": "split",
          "group": 1,
          "axis": "vertical",
          "id": "lower"
        },
        {
          "op": "move",
          "group": 2,
          "tab": "notes.md"
        },
        {
          "op": "open",
          "group": 1,
          "tab": "agent"
        }
      ],
      "expected": [
        [
          "files"
        ],
        [
          "agent"
        ],
        [
          "notes.md"
        ]
      ]
    },
    {
      "name": "closing the active tab selects the remaining tab",
      "steps": [
        {
          "op": "open",
          "group": 1,
          "tab": "a.md"
        },
        {
          "op": "open",
          "group": 1,
          "tab": "b.txt"
        },
        {
          "op": "remove",
          "tab": "b.txt"
        }
      ],
      "expected": [
        [
          "files"
        ],
        [
          "a.md"
        ]
      ]
    },
    {
      "name": "opening an existing tab does not duplicate it",
      "steps": [
        {
          "op": "open",
          "group": 1,
          "tab": "notes.md"
        },
        {
          "op": "open",
          "group": 1,
          "tab": "notes.md"
        },
        {
          "op": "move",
          "group": 0,
          "tab": "notes.md"
        }
      ],
      "expected": [
        [
          "files",
          "notes.md"
        ],
        []
      ]
    },
    {
      "name": "terminal tabs move without changing session identity",
      "steps": [
        {
          "op": "open",
          "group": 1,
          "tab": "terminal"
        },
        {
          "op": "split",
          "group": 1,
          "axis": "horizontal",
          "id": "terminal-right"
        },
        {
          "op": "move",
          "group": 2,
          "tab": "terminal"
        }
      ],
      "expected": [
        [
          "files"
        ],
        [],
        [
          "terminal"
        ]
      ]
    }
  ]
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/simple-contract/layout-cases.json"><ui:paste select="#simple-232e6fe962-0"/></ui:file>

</ui:doc>
