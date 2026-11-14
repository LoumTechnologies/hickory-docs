<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise shared workspace change boundaries

For maintainers changing workspace services independently of the editor renderer.

These shared data cases describe observable source outcomes and proposals that must not become edits. Each frontend consumes the same two-document Unicode rename and the same refusal cases. They establish planner behavior only; editor application, disk saving and native gestures need their own witnesses.

<ui:copy id="simple-shared-change-plan-cases">
{
  "sources": {
    "a.py": {
      "text": "é😀 one\nsecond",
      "writable": true
    },
    "b.py": {
      "text": "call(one)",
      "writable": true
    }
  },
  "edit": {
    "documentChanges": [
      {
        "textDocument": {
          "uri": "hick:///a.py",
          "version": null
        },
        "edits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 4
              },
              "end": {
                "line": 0,
                "character": 7
              }
            },
            "newText": "renamed"
          },
          {
            "range": {
              "start": {
                "line": 1,
                "character": 0
              },
              "end": {
                "line": 1,
                "character": 6
              }
            },
            "newText": "two"
          }
        ]
      },
      {
        "textDocument": {
          "uri": "hick:///b.py",
          "version": null
        },
        "edits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 5
              },
              "end": {
                "line": 0,
                "character": 8
              }
            },
            "newText": "renamed"
          }
        ]
      }
    ]
  },
  "after": {
    "a.py": "é😀 renamed\ntwo",
    "b.py": "call(renamed)"
  },
  "invalid": [
    {
      "name": "unknown source",
      "edit": {
        "changes": {
          "hick:///a.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 4
                },
                "end": {
                  "line": 0,
                  "character": 7
                }
              },
              "newText": "renamed"
            }
          ],
          "hick:///missing.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 5
                },
                "end": {
                  "line": 0,
                  "character": 8
                }
              },
              "newText": "renamed"
            }
          ]
        }
      }
    },
    {
      "name": "external identity",
      "edit": {
        "changes": {
          "file:///outside.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 5
                },
                "end": {
                  "line": 0,
                  "character": 8
                }
              },
              "newText": "renamed"
            }
          ]
        }
      }
    },
    {
      "name": "split Unicode scalar",
      "edit": {
        "changes": {
          "hick:///a.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 2
                },
                "end": {
                  "line": 0,
                  "character": 3
                }
              },
              "newText": "bad"
            }
          ]
        }
      }
    },
    {
      "name": "overlap",
      "edit": {
        "changes": {
          "hick:///a.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 4
                },
                "end": {
                  "line": 0,
                  "character": 7
                }
              },
              "newText": "renamed"
            },
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 5
                },
                "end": {
                  "line": 0,
                  "character": 7
                }
              },
              "newText": "overlap"
            }
          ]
        }
      }
    },
    {
      "name": "ambiguous insertions",
      "edit": {
        "changes": {
          "hick:///a.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 4
                },
                "end": {
                  "line": 0,
                  "character": 4
                }
              },
              "newText": "one"
            },
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 4
                },
                "end": {
                  "line": 0,
                  "character": 4
                }
              },
              "newText": "two"
            }
          ]
        }
      }
    },
    {
      "name": "backwards",
      "edit": {
        "changes": {
          "hick:///a.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 7
                },
                "end": {
                  "line": 0,
                  "character": 4
                }
              },
              "newText": "bad"
            }
          ]
        }
      }
    },
    {
      "name": "outside source",
      "edit": {
        "changes": {
          "hick:///a.py": [
            {
              "range": {
                "start": {
                  "line": 9,
                  "character": 0
                },
                "end": {
                  "line": 9,
                  "character": 1
                }
              },
              "newText": "bad"
            }
          ]
        }
      }
    },
    {
      "name": "virtual version",
      "edit": {
        "documentChanges": [
          {
            "textDocument": {
              "uri": "hick:///a.py",
              "version": 99
            },
            "edits": [
              {
                "range": {
                  "start": {
                    "line": 0,
                    "character": 4
                  },
                  "end": {
                    "line": 0,
                    "character": 7
                  }
                },
                "newText": "renamed"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "resource operation",
      "edit": {
        "documentChanges": [
          {
            "kind": "rename",
            "oldUri": "hick:///a.py",
            "newUri": "hick:///renamed.py"
          }
        ]
      }
    },
    {
      "name": "mixed representations",
      "edit": {
        "changes": {
          "hick:///a.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 4
                },
                "end": {
                  "line": 0,
                  "character": 7
                }
              },
              "newText": "renamed"
            }
          ]
        },
        "documentChanges": [
          {
            "textDocument": {
              "uri": "hick:///a.py",
              "version": null
            },
            "edits": [
              {
                "range": {
                  "start": {
                    "line": 0,
                    "character": 4
                  },
                  "end": {
                    "line": 0,
                    "character": 7
                  }
                },
                "newText": "renamed"
              },
              {
                "range": {
                  "start": {
                    "line": 1,
                    "character": 0
                  },
                  "end": {
                    "line": 1,
                    "character": 6
                  }
                },
                "newText": "two"
              }
            ]
          },
          {
            "textDocument": {
              "uri": "hick:///b.py",
              "version": null
            },
            "edits": [
              {
                "range": {
                  "start": {
                    "line": 0,
                    "character": 5
                  },
                  "end": {
                    "line": 0,
                    "character": 8
                  }
                },
                "newText": "renamed"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "read-only target",
      "readonly": "b.py",
      "edit": {
        "documentChanges": [
          {
            "textDocument": {
              "uri": "hick:///a.py",
              "version": null
            },
            "edits": [
              {
                "range": {
                  "start": {
                    "line": 0,
                    "character": 4
                  },
                  "end": {
                    "line": 0,
                    "character": 7
                  }
                },
                "newText": "renamed"
              },
              {
                "range": {
                  "start": {
                    "line": 1,
                    "character": 0
                  },
                  "end": {
                    "line": 1,
                    "character": 6
                  }
                },
                "newText": "two"
              }
            ]
          },
          {
            "textDocument": {
              "uri": "hick:///b.py",
              "version": null
            },
            "edits": [
              {
                "range": {
                  "start": {
                    "line": 0,
                    "character": 5
                  },
                  "end": {
                    "line": 0,
                    "character": 8
                  }
                },
                "newText": "renamed"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "malformed position",
      "edit": {
        "changes": {
          "hick:///a.py": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 0.5
                },
                "end": {
                  "line": 0,
                  "character": 3
                }
              },
              "newText": "bad"
            }
          ]
        }
      }
    }
  ]
}
</ui:copy>

<ui:file path="apps/simple-contract/change-plan-cases.json"><ui:paste select="#simple-shared-change-plan-cases"/></ui:file>
</ui:doc>
