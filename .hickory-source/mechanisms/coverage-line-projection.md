<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Project measured coverage through captured source ownership

For maintainers changing coverage marks independently of their editor renderer.

A generated location and an authored line are different units. Several measured runtime lines can map to one authored line, so retain mixed hit/miss evidence and its original locations. A missing owner fingerprint or an observed execution-source change grants no editor mark. Missing or inexact count quantities remain null, while a reported hit/miss can remain known. These projections do not verify a test outcome or freeze execution inputs.

## Project source-bound locations on browser

Keep runtime rows and compiler-reported authored rows under their own captured owner. Aggregate hit and miss independently, use the minimum count only when every contributing count is known, and preserve each origin. Counts beyond the shared exact-integer display range are unavailable quantities, never rounded proof. Validate line numbers, and do not reinterpret an unbound file as zero coverage.

<ui:copy id="coverage-projection-browser">
export interface CoverageProjectionInput {
  id:string;source_changed_during_run?:boolean;identity:{source_hashes:Record<string,string>};
  evidence?:{report:{files:Record<string,{line_counts:boolean;lines:Record<string,number>}>}}|null;
  mappings?:Record<string,Record<string,{path:string;line:number}>>;
}
export interface CoverageProjectedLine {line:number;state:"covered"|"uncovered"|"mixed";count:number|null;origins:string[]}
/** One authored line may represent several runtime locations; retain mixed evidence. */
export function coverageProjection(record:CoverageProjectionInput):Record<string,CoverageProjectedLine[]> {
  if(record.source_changed_during_run!==false)return {};
  type Aggregate={hit:boolean;miss:boolean;count:number|null;origins:string[]};
  const owners=new Map<string,Map<number,Aggregate>>();
  const add=(path:string,line:number,count:number,available:boolean,origin:string)=>{
    if(!/^[0-9a-f]{64}$/i.test(record.identity.source_hashes[path]??"")||!Number.isInteger(line)||line<1||line>2147483647||!Number.isFinite(count)||count<0)return;
    const rows=owners.get(path)??new Map<number,Aggregate>();owners.set(path,rows);
    const known=available&&Number.isSafeInteger(count)?count:null,old=rows.get(line-1);
    rows.set(line-1,{hit:(old?.hit??false)||count>0,miss:(old?.miss??false)||count===0,count:old?(old.count===null||known===null?null:Math.min(old.count,known)):known,origins:[...(old?.origins??[]),origin]});
  };
  for(const [path,file] of Object.entries(record.evidence?.report.files??{}))for(const [value,count] of Object.entries(file.lines)){
    const line=Number(value),origin=`${path}:${value}`;add(path,line,count,file.line_counts,origin);
    const mapping=record.mappings?.[path]?.[value];if(mapping&&mapping.path!==path)add(mapping.path,mapping.line,count,file.line_counts,origin);
  }
  return Object.fromEntries([...owners].map(([path,rows])=>[path,[...rows].sort(([a],[b])=>a-b).map(([line,row])=>({line,state:row.hit&&row.miss?"mixed" as const:row.hit?"covered" as const:"uncovered" as const,count:row.count,origins:row.origins.sort()}))]));
}
</ui:copy>

The same bundled fixture checks mixed generated consequences, binary-only counts, excessive count quantities, unknown fingerprints, observed/missing source-change information and invalid mapped positions. These are projection witnesses, not a simulated collector.

<ui:copy id="coverage-projection-browser-witness">
import {expect,it} from "vitest";
import fixture from "../../../simple-contract/coverage-projections.json";
import {coverageProjection,type CoverageProjectionInput} from "./CoverageProjection";
for(const test of fixture.cases)it(test.name,()=>{expect(coverageProjection(test.record as unknown as CoverageProjectionInput)).toEqual(test.expected);});
</ui:copy>

## Project source-bound locations on apple

Keep runtime rows and compiler-reported authored rows under their own captured owner. Aggregate hit and miss independently, use the minimum count only when every contributing count is known, and preserve each origin. Counts beyond the shared exact-integer display range are unavailable quantities, never rounded proof. Validate line numbers, and do not reinterpret an unbound file as zero coverage.

<ui:copy id="coverage-projection-apple">
import Foundation
struct CoverageProjectionRecord:Decodable {
    struct Identity:Decodable {var source_hashes:[String:String]}
    struct File:Decodable {var line_counts:Bool;var lines:[String:UInt64]}
    struct Report:Decodable {var files:[String:File]}
    struct Evidence:Decodable {var report:Report}
    struct Mapping:Decodable {var path:String;var line:Int}
    var id:String;var source_changed_during_run:Bool?;var identity:Identity;var evidence:Evidence?;var mappings:[String:[String:Mapping]]?
}
struct CoverageProjectedLine:Decodable,Equatable {
    var line:Int;var state:String;var count:UInt64?;var origins:[String]
}
func coverageProjection(_ record:CoverageProjectionRecord)->[String:[CoverageProjectedLine]] {
    guard record.source_changed_during_run==false else {return [:]}
    struct Aggregate {var hit:Bool;var miss:Bool;var count:UInt64?;var origins:[String]}
    var owners:[String:[Int:Aggregate]]=[:]
    func add(_ path:String,_ line:Int,_ count:UInt64,_ available:Bool,_ origin:String){
        guard let hash=record.identity.source_hashes[path],hash.count==64,hash.allSatisfy({$0.isASCII && $0.isHexDigit}),line>0,line<=Int(Int32.max) else {return}
        let known:UInt64?=available && count<=9_007_199_254_740_991 ? count:nil
        let old=owners[path]?[line-1]
        let minimum:UInt64?
        if let old {if let left=old.count,let right=known {minimum=min(left,right)}else{minimum=nil}}
        else {minimum=known}
        owners[path,default:[:]][line-1]=Aggregate(hit:(old?.hit ?? false)||count>0,miss:(old?.miss ?? false)||count==0,count:minimum,origins:(old?.origins ?? [])+[origin])
    }
    for (path,file) in record.evidence?.report.files ?? [:] {
        for (value,count) in file.lines {
            guard let line=Int(value) else {continue};let origin="\(path):\(value)";add(path,line,count,file.line_counts,origin)
            if let mapping=record.mappings?[path]?[value],mapping.path != path {add(mapping.path,mapping.line,count,file.line_counts,origin)}
        }
    }
    return owners.mapValues {rows in rows.keys.sorted().map {line in let value=rows[line]!;return CoverageProjectedLine(line:line,state:value.hit && value.miss ? "mixed":value.hit ? "covered":"uncovered",count:value.count,origins:value.origins.sorted())}}
}
</ui:copy>

The same bundled fixture checks mixed generated consequences, binary-only counts, excessive count quantities, unknown fingerprints, observed/missing source-change information and invalid mapped positions. These are projection witnesses, not a simulated collector.

<ui:copy id="coverage-projection-apple-witness">
import XCTest
@testable import HickorySimple
final class CoverageProjectionTests:XCTestCase {
    struct Fixture:Decodable {var version:Int;var cases:[Case]}
    struct Case:Decodable {var name:String;var record:CoverageProjectionRecord;var expected:[String:[CoverageProjectedLine]]}
    func testBundledCoverageOwnershipAndUnknowns()throws {
        let url=try XCTUnwrap(simpleResource("coverage-projections",withExtension:"json"),"Rebuild the shared capability resources.")
        let fixture=try JSONDecoder().decode(Fixture.self,from:Data(contentsOf:url));XCTAssertEqual(fixture.version,1);XCTAssertEqual(fixture.cases.count,5)
        for test in fixture.cases {XCTAssertEqual(coverageProjection(test.record),test.expected,test.name)}
    }
}
</ui:copy>

## Project source-bound locations on android

Keep runtime rows and compiler-reported authored rows under their own captured owner. Aggregate hit and miss independently, use the minimum count only when every contributing count is known, and preserve each origin. Counts beyond the shared exact-integer display range are unavailable quantities, never rounded proof. Validate line numbers, and do not reinterpret an unbound file as zero coverage.

<ui:copy id="coverage-projection-android">
package com.hickorydocs.simple
import org.json.JSONObject

data class CoverageProjectedLine(val line:Int,val state:String,val count:Long?,val origins:List<String>)
fun coverageProjection(record:JSONObject):Map<String,List<CoverageProjectedLine>> {
    if(record.opt("source_changed_during_run")!=false)return emptyMap()
    data class Aggregate(val hit:Boolean,val miss:Boolean,val count:Long?,val origins:List<String>)
    val hashes=record.optJSONObject("identity")?.optJSONObject("source_hashes") ?: return emptyMap()
    val owners=mutableMapOf<String,MutableMap<Int,Aggregate>>()
    fun add(path:String,line:Int,countText:String,available:Boolean,origin:String){
        if(!hashes.optString(path).matches(Regex("[0-9a-fA-F]{64}"))||line<1||countText.isEmpty()||!countText.all {it in '0'..'9'})return
        val hit=countText.any {it!='0'};val parsed=countText.toLongOrNull();val known=parsed?.takeIf {available&&it<=9_007_199_254_740_991}
        val rows=owners.getOrPut(path){mutableMapOf()};val old=rows[line-1]
        rows[line-1]=Aggregate(old?.hit==true||hit,old?.miss==true||!hit,if(old==null)known else if(old.count==null||known==null)null else minOf(old.count,known),old?.origins.orEmpty()+origin)
    }
    val files=record.optJSONObject("evidence")?.optJSONObject("report")?.optJSONObject("files") ?: return emptyMap()
    val mappings=record.optJSONObject("mappings")
    for(path in files.keys()){
        val file=files.getJSONObject(path);val lines=file.getJSONObject("lines")
        for(value in lines.keys()){
            val line=value.toIntOrNull() ?: continue;val count=lines.get(value).toString();val available=file.optBoolean("line_counts");val origin="$path:$value"
            add(path,line,count,available,origin)
            val mapping=mappings?.optJSONObject(path)?.optJSONObject(value)
            if(mapping!=null&&mapping.optString("path")!=path)add(mapping.getString("path"),mapping.optInt("line"),count,available,origin)
        }
    }
    return owners.mapValues {(_,rows)->rows.keys.sorted().map {line->val value=rows.getValue(line);CoverageProjectedLine(line,if(value.hit&&value.miss)"mixed" else if(value.hit)"covered" else "uncovered",value.count,value.origins.sorted())}}
}
</ui:copy>

The same bundled fixture checks mixed generated consequences, binary-only counts, excessive count quantities, unknown fingerprints, observed/missing source-change information and invalid mapped positions. These are projection witnesses, not a simulated collector.

<ui:copy id="coverage-projection-android-witness">
package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
class CoverageProjectionTest {
    @Test fun bundledCoverageOwnershipAndUnknowns(){
        val fixture=JSONObject(javaClass.classLoader!!.getResourceAsStream("coverage-projections.json")!!.bufferedReader().use {it.readText()});assertEquals(1,fixture.getInt("version"));val cases=fixture.getJSONArray("cases");assertEquals(5,cases.length())
        for(index in 0 until cases.length()){
            val test=cases.getJSONObject(index);val raw=test.getJSONObject("expected")
            val expected=raw.keys().asSequence().associateWith {path->val rows=raw.getJSONArray(path);(0 until rows.length()).map {i->val row=rows.getJSONObject(i);val origins=row.getJSONArray("origins");CoverageProjectedLine(row.getInt("line"),row.getString("state"),if(row.isNull("count"))null else row.getLong("count"),(0 until origins.length()).map {origins.getString(it)})}}
            assertEquals(test.getString("name"),expected,coverageProjection(test.getJSONObject("record")))
        }
    }
}
</ui:copy>

## Resolve native capability assets

The native witness and upcoming shared tool surfaces must load bundled resources, not silently fall back to editable workspace files. Resolve the package and application bundle layouts used by the current builds; a missing resource stays unavailable.

<ui:copy id="simple-bundled-capability-resource">
import Foundation
private class SimpleResourceMarker {}
/** Resolve only bundled capability assets; no source workspace is searched. */
func simpleResource(_ name:String,withExtension ext:String)->URL? {
    guard !name.isEmpty,!ext.isEmpty,!name.contains("/"),!name.contains("\\"),name != ".",name != "..",!ext.contains("/"),!ext.contains("\\") else {return nil}
    let owner=Bundle(for:SimpleResourceMarker.self)
    let roots=[Bundle.main.resourceURL,Bundle.main.bundleURL,owner.bundleURL.deletingLastPathComponent(),URL(fileURLWithPath:CommandLine.arguments[0]).deletingLastPathComponent()].compactMap {$0}
    var bundles=[Bundle.main,owner]+roots.compactMap {Bundle(url:$0.appendingPathComponent("HickorySimple_HickorySimple.bundle"))}
    #if SWIFT_PACKAGE
    bundles.append(Bundle.module)
    #endif
    for bundle in bundles {
        if let url=bundle.url(forResource:name,withExtension:ext,subdirectory:"HickoryAssets/Resources") ?? bundle.url(forResource:name,withExtension:ext,subdirectory:"Resources") ?? bundle.url(forResource:name,withExtension:ext),url.isFileURL {return url}
    }
    return nil
}
</ui:copy>

## Witness the same observations on every platform

The first case deliberately combines a covered and uncovered generated line with a binary-only measurement on one authored line. Its count must be unknown and its state mixed. Other cases retire marks after changed/unobserved execution source, remove an unbound authored owner, or reject invalid mapped line numbers. Both native builds consume this same fixture as a real resource.

<ui:copy id="coverage-projection-shared-fixture">
{
  "version": 1,
  "cases": [
    {
      "name": "mixed generated consequences and unavailable quantities",
      "record": {
        "id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "source_changed_during_run": false,
        "identity": {
          "source_hashes": {
            "generated.py": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "second.py": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "note.md": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "huge.py": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "bad.py": "unbound"
          }
        },
        "evidence": {
          "report": {
            "files": {
              "generated.py": {
                "line_counts": true,
                "lines": {
                  "1": 3,
                  "2": 0
                }
              },
              "second.py": {
                "line_counts": false,
                "lines": {
                  "1": 0
                }
              },
              "huge.py": {
                "line_counts": true,
                "lines": {
                  "1": 9007199254740992
                }
              },
              "unknown.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              },
              "bad.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              }
            }
          }
        },
        "mappings": {
          "generated.py": {
            "1": {
              "path": "note.md",
              "line": 4
            },
            "2": {
              "path": "note.md",
              "line": 4
            }
          },
          "second.py": {
            "1": {
              "path": "note.md",
              "line": 4
            }
          }
        }
      },
      "expected": {
        "generated.py": [
          {
            "line": 0,
            "state": "covered",
            "count": 3,
            "origins": [
              "generated.py:1"
            ]
          },
          {
            "line": 1,
            "state": "uncovered",
            "count": 0,
            "origins": [
              "generated.py:2"
            ]
          }
        ],
        "second.py": [
          {
            "line": 0,
            "state": "uncovered",
            "count": null,
            "origins": [
              "second.py:1"
            ]
          }
        ],
        "note.md": [
          {
            "line": 3,
            "state": "mixed",
            "count": null,
            "origins": [
              "generated.py:1",
              "generated.py:2",
              "second.py:1"
            ]
          }
        ],
        "huge.py": [
          {
            "line": 0,
            "state": "covered",
            "count": null,
            "origins": [
              "huge.py:1"
            ]
          }
        ]
      }
    },
    {
      "name": "observed execution source changed",
      "record": {
        "id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "source_changed_during_run": true,
        "identity": {
          "source_hashes": {
            "generated.py": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "second.py": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "note.md": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "huge.py": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "bad.py": "unbound"
          }
        },
        "evidence": {
          "report": {
            "files": {
              "generated.py": {
                "line_counts": true,
                "lines": {
                  "1": 3,
                  "2": 0
                }
              },
              "second.py": {
                "line_counts": false,
                "lines": {
                  "1": 0
                }
              },
              "huge.py": {
                "line_counts": true,
                "lines": {
                  "1": 9007199254740992
                }
              },
              "unknown.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              },
              "bad.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              }
            }
          }
        },
        "mappings": {
          "generated.py": {
            "1": {
              "path": "note.md",
              "line": 4
            },
            "2": {
              "path": "note.md",
              "line": 4
            }
          },
          "second.py": {
            "1": {
              "path": "note.md",
              "line": 4
            }
          }
        }
      },
      "expected": {}
    },
    {
      "name": "missing change observation",
      "record": {
        "id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "identity": {
          "source_hashes": {
            "generated.py": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "second.py": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "note.md": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "huge.py": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "bad.py": "unbound"
          }
        },
        "evidence": {
          "report": {
            "files": {
              "generated.py": {
                "line_counts": true,
                "lines": {
                  "1": 3,
                  "2": 0
                }
              },
              "second.py": {
                "line_counts": false,
                "lines": {
                  "1": 0
                }
              },
              "huge.py": {
                "line_counts": true,
                "lines": {
                  "1": 9007199254740992
                }
              },
              "unknown.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              },
              "bad.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              }
            }
          }
        },
        "mappings": {
          "generated.py": {
            "1": {
              "path": "note.md",
              "line": 4
            },
            "2": {
              "path": "note.md",
              "line": 4
            }
          },
          "second.py": {
            "1": {
              "path": "note.md",
              "line": 4
            }
          }
        }
      },
      "expected": {}
    },
    {
      "name": "unbound authored owner",
      "record": {
        "id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "source_changed_during_run": false,
        "identity": {
          "source_hashes": {
            "generated.py": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "second.py": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "huge.py": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "bad.py": "unbound"
          }
        },
        "evidence": {
          "report": {
            "files": {
              "generated.py": {
                "line_counts": true,
                "lines": {
                  "1": 3,
                  "2": 0
                }
              },
              "second.py": {
                "line_counts": false,
                "lines": {
                  "1": 0
                }
              },
              "huge.py": {
                "line_counts": true,
                "lines": {
                  "1": 9007199254740992
                }
              },
              "unknown.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              },
              "bad.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              }
            }
          }
        },
        "mappings": {
          "generated.py": {
            "1": {
              "path": "note.md",
              "line": 4
            },
            "2": {
              "path": "note.md",
              "line": 4
            }
          },
          "second.py": {
            "1": {
              "path": "note.md",
              "line": 4
            }
          }
        }
      },
      "expected": {
        "generated.py": [
          {
            "line": 0,
            "state": "covered",
            "count": 3,
            "origins": [
              "generated.py:1"
            ]
          },
          {
            "line": 1,
            "state": "uncovered",
            "count": 0,
            "origins": [
              "generated.py:2"
            ]
          }
        ],
        "second.py": [
          {
            "line": 0,
            "state": "uncovered",
            "count": null,
            "origins": [
              "second.py:1"
            ]
          }
        ],
        "huge.py": [
          {
            "line": 0,
            "state": "covered",
            "count": null,
            "origins": [
              "huge.py:1"
            ]
          }
        ]
      }
    },
    {
      "name": "invalid mapped positions",
      "record": {
        "id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "source_changed_during_run": false,
        "identity": {
          "source_hashes": {
            "generated.py": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "second.py": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "note.md": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "huge.py": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "bad.py": "unbound"
          }
        },
        "evidence": {
          "report": {
            "files": {
              "generated.py": {
                "line_counts": true,
                "lines": {
                  "1": 3,
                  "2": 0
                }
              },
              "second.py": {
                "line_counts": false,
                "lines": {
                  "1": 0
                }
              },
              "huge.py": {
                "line_counts": true,
                "lines": {
                  "1": 9007199254740992
                }
              },
              "unknown.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              },
              "bad.py": {
                "line_counts": true,
                "lines": {
                  "1": 0
                }
              }
            }
          }
        },
        "mappings": {
          "generated.py": {
            "1": {
              "path": "note.md",
              "line": 0
            },
            "2": {
              "path": "note.md",
              "line": 2147483648
            }
          },
          "second.py": {
            "1": {
              "path": "note.md",
              "line": -1
            }
          }
        }
      },
      "expected": {
        "generated.py": [
          {
            "line": 0,
            "state": "covered",
            "count": 3,
            "origins": [
              "generated.py:1"
            ]
          },
          {
            "line": 1,
            "state": "uncovered",
            "count": 0,
            "origins": [
              "generated.py:2"
            ]
          }
        ],
        "second.py": [
          {
            "line": 0,
            "state": "uncovered",
            "count": null,
            "origins": [
              "second.py:1"
            ]
          }
        ],
        "huge.py": [
          {
            "line": 0,
            "state": "covered",
            "count": null,
            "origins": [
              "huge.py:1"
            ]
          }
        ]
      }
    }
  ]
}
</ui:copy>

<ui:file path="apps/web/src/coverage/CoverageProjection.ts"><ui:paste select="#coverage-projection-browser"/></ui:file>
<ui:file path="apps/web/src/coverage/CoverageProjection.test.ts"><ui:paste select="#coverage-projection-browser-witness"/></ui:file>
<ui:file path="apps/apple-simple/Sources/CoverageProjection.swift"><ui:paste select="#coverage-projection-apple"/></ui:file>
<ui:file path="apps/apple-simple/Tests/CoverageProjectionTests.swift"><ui:paste select="#coverage-projection-apple-witness"/></ui:file>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/CoverageProjection.kt"><ui:paste select="#coverage-projection-android"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/CoverageProjectionTest.kt"><ui:paste select="#coverage-projection-android-witness"/></ui:file>
<ui:file path="apps/apple-simple/Sources/SimpleResource.swift"><ui:paste select="#simple-bundled-capability-resource"/></ui:file>
<ui:file path="apps/simple-contract/coverage-projections.json"><ui:paste select="#coverage-projection-shared-fixture"/></ui:file>
<ui:file path="apps/apple-simple/Sources/Resources/coverage-projections.json"><ui:paste select="#coverage-projection-shared-fixture"/></ui:file>
</ui:doc>
