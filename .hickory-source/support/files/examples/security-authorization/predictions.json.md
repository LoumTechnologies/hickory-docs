<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/security-authorization/predictions.json

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3111069cf15a8a14c7d5748bf4542415752783487b9003953d4f7d0f547cf94b`.

<book:entry path="examples/security-authorization/predictions.json">
{
  "format": "hickory-selected-prediction-history",
  "version": 1,
  "selected_scenarios": [
    "a29ea55609ad0e7597cb3f6245b2a539b08d5119dc3627da58f26af230c32ca2"
  ],
  "omitted_events": 0,
  "omitted_attempts": 0,
  "omitted_context_observations": 0,
  "history": {
    "generation": 7,
    "attempts": [
      {
        "attempt": "d1c0b649-865f-45c1-9254-18d64870fc3e",
        "scenario": "a29ea55609ad0e7597cb3f6245b2a539b08d5119dc3627da58f26af230c32ca2",
        "seal": "3bb49723-40cb-44e0-90b1-d02b21bf8dec",
        "classification": "execution_rejected",
        "prediction_matched": false,
        "original_requirement_satisfied": false,
        "observed": false,
        "promoted": false,
        "current_certification": false
      },
      {
        "attempt": "c461bc9c-a8e5-4233-b27a-373dd7d85668",
        "scenario": "a29ea55609ad0e7597cb3f6245b2a539b08d5119dc3627da58f26af230c32ca2",
        "seal": "79bbe31e-465e-41b6-b3bc-71ba4579bed1",
        "classification": "verified_after_correction",
        "prediction_matched": true,
        "original_requirement_satisfied": true,
        "observed": false,
        "promoted": true,
        "current_certification": false
      }
    ],
    "ordering": [
      {
        "digest": "64a9225ea2f6de72cce46f9380d2044049217f9183484e370a7e0237f4977acc",
        "event": {
          "actor": {
            "id": "synthetic-authorization-predictor",
            "kind": "agent"
          },
          "context": [],
          "id": "3bb49723-40cb-44e0-90b1-d02b21bf8dec",
          "kind": "sealed",
          "seal": {
            "identity": {
              "dependencies": {},
              "environment": {
                "adapter_python": "ff37985de8999bf5344ea3d8ba4910e3ebfc3b4488b253dd3e51a40168bf0687",
                "adapter_recipe": "7af3bb7cf04ba2a5117515cb5afdbf00b6f40c2d3949836d0654c6a97970a1c8",
                "output_python": "571c6fc4670bbc08a4c298bb4483a676eb4044ef2f6bf4c331e2feacb923db74"
              },
              "inputs": "14fd478a3c4ab632c0a6eef2da892bbd0f04486b75cd3cd6239d65eaac224a1f",
              "source": "20a184766b132a6f783e162d42546b70986c5d4c46807bd39d37a5728f87ffd9"
            },
            "predicted": {
              "kind": "fields",
              "value": {
                "debugger": {
                  "kind": "equals",
                  "value": 0
                },
                "output": {
                  "kind": "equals",
                  "value": 0
                }
              }
            },
            "rationale": "Cross-tenant access must be denied while legitimate access stays available",
            "scenario": {
              "family": "synthetic-cross-tenant-authorization",
              "identity_known": true,
              "inputs": {
                "record": "record-b",
                "records": {
                  "record-a": "tenant-a",
                  "record-b": "tenant-b"
                },
                "tenant": "tenant-a"
              },
              "requirement": {
                "kind": "fields",
                "value": {
                  "debugger": {
                    "kind": "equals",
                    "value": 0
                  },
                  "output": {
                    "kind": "equals",
                    "value": 0
                  }
                }
              },
              "requirement_reference": "R-AUTH-1; R-LEGIT-1"
            },
            "target": {
              "debugger": {
                "end_byte": 733,
                "expression": "allowed",
                "frame": "authorize",
                "hit": 1,
                "kind": "debugger",
                "source": "root",
                "source_revision": "20a184766b132a6f783e162d42546b70986c5d4c46807bd39d37a5728f87ffd9",
                "start_byte": 714
              },
              "kind": "combined",
              "output": {
                "execution": "c:24:json",
                "kind": "output"
              }
            }
          }
        },
        "previous": "",
        "sequence": 1
      },
      {
        "digest": "76c2d083e554db801ffbc7a930f8a929c22b8c9fd0232c8aa14c0b4a96dee5c1",
        "event": {
          "actor": {
            "id": "synthetic-authorization-predictor",
            "kind": "agent"
          },
          "id": "d1c0b649-865f-45c1-9254-18d64870fc3e",
          "kind": "started",
          "replay": false,
          "seal": "3bb49723-40cb-44e0-90b1-d02b21bf8dec"
        },
        "previous": "64a9225ea2f6de72cce46f9380d2044049217f9183484e370a7e0237f4977acc",
        "sequence": 2
      },
      {
        "digest": "eb63bcd35a56a6578e23f3112484f37811eed1a9b797de31b18ca0771de67599",
        "event": {
          "actual": "withheld from agents",
          "attempt": "d1c0b649-865f-45c1-9254-18d64870fc3e",
          "kind": "finished"
        },
        "previous": "76c2d083e554db801ffbc7a930f8a929c22b8c9fd0232c8aa14c0b4a96dee5c1",
        "sequence": 3
      },
      {
        "digest": "08837ca2c313572170292b9258df1e96b89871b8e2984edb397d93018153b251",
        "event": {
          "actor": {
            "id": "synthetic-authorization-predictor",
            "kind": "agent"
          },
          "context": [],
          "id": "79bbe31e-465e-41b6-b3bc-71ba4579bed1",
          "kind": "sealed",
          "seal": {
            "identity": {
              "dependencies": {},
              "environment": {
                "adapter_python": "ff37985de8999bf5344ea3d8ba4910e3ebfc3b4488b253dd3e51a40168bf0687",
                "adapter_recipe": "7af3bb7cf04ba2a5117515cb5afdbf00b6f40c2d3949836d0654c6a97970a1c8",
                "output_python": "571c6fc4670bbc08a4c298bb4483a676eb4044ef2f6bf4c331e2feacb923db74"
              },
              "inputs": "14fd478a3c4ab632c0a6eef2da892bbd0f04486b75cd3cd6239d65eaac224a1f",
              "source": "b60f7ed89deae86c9026c7dfe1535e68f3f39d4cf8fb37fd186876f33db12102"
            },
            "predicted": {
              "kind": "fields",
              "value": {
                "debugger": {
                  "kind": "equals",
                  "value": 0
                },
                "output": {
                  "kind": "equals",
                  "value": 0
                }
              }
            },
            "rationale": "Cross-tenant access must be denied while legitimate access stays available",
            "scenario": {
              "family": "synthetic-cross-tenant-authorization",
              "identity_known": true,
              "inputs": {
                "record": "record-b",
                "records": {
                  "record-a": "tenant-a",
                  "record-b": "tenant-b"
                },
                "tenant": "tenant-a"
              },
              "requirement": {
                "kind": "fields",
                "value": {
                  "debugger": {
                    "kind": "equals",
                    "value": 0
                  },
                  "output": {
                    "kind": "equals",
                    "value": 0
                  }
                }
              },
              "requirement_reference": "R-AUTH-1; R-LEGIT-1"
            },
            "target": {
              "debugger": {
                "end_byte": 763,
                "expression": "allowed",
                "frame": "authorize",
                "hit": 1,
                "kind": "debugger",
                "source": "root",
                "source_revision": "b60f7ed89deae86c9026c7dfe1535e68f3f39d4cf8fb37fd186876f33db12102",
                "start_byte": 744
              },
              "kind": "combined",
              "output": {
                "execution": "c:24:json",
                "kind": "output"
              }
            }
          }
        },
        "previous": "eb63bcd35a56a6578e23f3112484f37811eed1a9b797de31b18ca0771de67599",
        "sequence": 4
      },
      {
        "digest": "ea87a5ed1af993f597fd1fe855d1a1d3f08db2a713737ffc1c70cb90e23186cc",
        "event": {
          "actor": {
            "id": "synthetic-authorization-predictor",
            "kind": "agent"
          },
          "id": "c461bc9c-a8e5-4233-b27a-373dd7d85668",
          "kind": "started",
          "replay": false,
          "seal": "79bbe31e-465e-41b6-b3bc-71ba4579bed1"
        },
        "previous": "08837ca2c313572170292b9258df1e96b89871b8e2984edb397d93018153b251",
        "sequence": 5
      },
      {
        "digest": "9ed6b0db2a26ff39570953054f12bb941527e792915215aa7382d42ec83a9c30",
        "event": {
          "actual": "withheld from agents",
          "attempt": "c461bc9c-a8e5-4233-b27a-373dd7d85668",
          "kind": "finished"
        },
        "previous": "ea87a5ed1af993f597fd1fe855d1a1d3f08db2a713737ffc1c70cb90e23186cc",
        "sequence": 6
      },
      {
        "digest": "9e6b52df735e7acd5365a62ade08ec622d6c7d33d0521fee8abb8af9d5cabce7",
        "event": {
          "actor": {
            "id": "synthetic-authorization-predictor",
            "kind": "agent"
          },
          "attempt": "c461bc9c-a8e5-4233-b27a-373dd7d85668",
          "identity": {
            "dependencies": {},
            "environment": {
              "adapter_python": "ff37985de8999bf5344ea3d8ba4910e3ebfc3b4488b253dd3e51a40168bf0687",
              "adapter_recipe": "7af3bb7cf04ba2a5117515cb5afdbf00b6f40c2d3949836d0654c6a97970a1c8",
              "output_python": "571c6fc4670bbc08a4c298bb4483a676eb4044ef2f6bf4c331e2feacb923db74"
            },
            "inputs": "14fd478a3c4ab632c0a6eef2da892bbd0f04486b75cd3cd6239d65eaac224a1f",
            "source": "b60f7ed89deae86c9026c7dfe1535e68f3f39d4cf8fb37fd186876f33db12102"
          },
          "kind": "promoted"
        },
        "previous": "9ed6b0db2a26ff39570953054f12bb941527e792915215aa7382d42ec83a9c30",
        "sequence": 7
      }
    ]
  },
  "debugger_contexts": [
    {
      "adapter": "combined_live_v1",
      "attempt": "c461bc9c-a8e5-4233-b27a-373dd7d85668",
      "file": "authorization.py",
      "hits": [
        {
          "context": {
            "breakpoint_bound": true,
            "document_line": 15,
            "frame": 2,
            "function": "authorize",
            "run": "c4a7e875-7338-4227-86e2-a212616a6a69",
            "source": "/tmp/.tmpMLCemK/authorization.py",
            "source_line": 3,
            "thread": 1
          },
          "index": 1
        }
      ],
      "line": 4,
      "moved": false,
      "problem": false,
      "truncated": true
    },
    {
      "adapter": "combined_live_v1",
      "attempt": "d1c0b649-865f-45c1-9254-18d64870fc3e",
      "file": "authorization.py",
      "hits": [
        {
          "context": {
            "breakpoint_bound": true,
            "document_line": 15,
            "frame": 2,
            "function": "authorize",
            "run": "12e33a8a-64dc-44c4-970e-62cbd47f052f",
            "source": "/tmp/.tmpoDgVeC/authorization.py",
            "source_line": 3,
            "thread": 1
          },
          "index": 1
        }
      ],
      "line": 4,
      "moved": false,
      "problem": false,
      "truncated": true
    }
  ],
  "scope": "Selected historical host declarations; actual values and private evidence withheld. No current-source certification, human review, upload or authenticity is established by serialization."
}
</book:entry>

</book:doc>
