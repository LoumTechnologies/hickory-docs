import Foundation
import XCTest

@testable import HickorySimple

final class SnippetTransformV3Tests: XCTestCase {
  func load(_ name: String) throws -> Any {
    let names = [
      "case-17.json": "snippet-regex-case17.json",
      "oracle-cases-compiled.json": "snippet-regex-pattern-cases.json",
      "format-oracle-cases-compiled.json": "snippet-regex-format-cases.json",
      "budget-cases.json": "snippet-regex-budget-cases.json",
    ]
    let base = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
      .deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent(
        "simple-contract")
    return try JSONSerialization.jsonObject(
      with: Data(contentsOf: base.appendingPathComponent(names[name]!)))
  }
  func testIndependentPatternsFormatsAndBudgetRefusals() throws {
    let policy = try RegexV3CaseData(load("case-17.json") as! [String: Any])
    let rows = try load("oracle-cases-compiled.json") as! [[String: Any]]
    var passed = 0
    var failed = 0
    for row in rows {
      let test = row["case"] as! [String: Any]
      do {
        if test["error"] as? Bool == true {
          guard row["compile_error"] is String else {
            throw EngineError(message: "Invalid regex compiled")
          }
          passed += 1
          continue
        }
        let program = try readRegexV3(row["program"]!)
        let input = test["input"] as! String
        var machine = try RegexV3Machine(input, program, policy)
        let result = try machine.find(program).map { value -> [String: Any] in
          [
            "from": machine.offsets[value.from], "to": machine.offsets[value.to],
            "indices": value.captures.map { range -> Any in
              range.map { [machine.offsets[$0.lowerBound], machine.offsets[$0.upperBound]] } as Any?
                ?? NSNull()
            },
            "capture_units": value.captures.map { range -> Any in
              range.map { machine.units($0) } as Any? ?? NSNull()
            },
          ]
        }
        let expected = (test["matches"] as! [[String: Any]]).map { value -> [String: Any] in
          var value = value
          value.removeValue(forKey: "name_units")
          return value
        }
        let actual = try JSONSerialization.data(withJSONObject: result, options: .sortedKeys)
        let golden = try JSONSerialization.data(withJSONObject: expected, options: .sortedKeys)
        guard actual == golden else {
          throw EngineError(
            message: "Capture/match mismatch: " + String(data: actual, encoding: .utf8)!)
        }
        passed += 1
      } catch {
        failed += 1
        print("FAIL \(test["name"]!) \(error)")
      }
    }
    print("Swift v3 patterns \(passed) PASS \(failed) FAIL")
    XCTAssertEqual(failed, 0)
    let formats = try load("format-oracle-cases-compiled.json") as! [[String: Any]]
    var formatPassed = 0
    for row in formats {
      let test = row["case"] as! [String: Any]
      do {
        let program = try readRegexV3(row["program"]!)
        var work = 0
        let actual = try applyRegexV3(program, test["input"] as! String, policy, work: &work)
        guard actual == test["output"] as! String else {
          throw EngineError(message: "Expected \(test["output"]!), got \(actual)")
        }
        formatPassed += 1
      } catch {
        failed += 1
        print("FORMAT FAIL \(test["source"]!) \(error)")
      }
    }
    print("Swift v3 formats \(formatPassed) PASS \(failed) FAIL")
    XCTAssertEqual(failed, 0)
    let budgets = (try load("budget-cases.json") as! [String: Any])["cases"] as! [[String: Any]]
    var budgetPassed = 0
    for row in budgets {
      do {
        let program = try readRegexV3(row["program"]!)
        if row["phase"] as? String == "run" {
          var work = row["work"] as! Int
          _ = try applyRegexV3(program, row["input"] as! String, policy, work: &work)
        }
        print("BUDGET FAIL \(row["name"]!) accepted")
        failed += 1
      } catch { budgetPassed += 1 }
    }
    print("Swift v3 budget/refusal \(budgetPassed) PASS \(failed) FAIL")
    XCTAssertEqual(failed, 0)

    XCTAssertEqual(passed, 858)
    XCTAssertEqual(formatPassed, 23)
    XCTAssertEqual(budgetPassed, 23)
  }
  func testCanonicalNumericMetadataAndOuterInstructionBudget() throws {
    let rows = try load("budget-cases.json") as! [String: Any]
    var raw = ((rows["cases"] as! [[String: Any]])[0]["program"] as! [String: Any])
    raw["case_map"] = "hickory-ucd-17.0.0-v1"
    raw["version"] = 3.0
    raw["groups"] = 0.0
    raw["flags"] = "u"
    let ranges = (0..<55000).map { [Double($0 * 16), Double($0 * 16)] }
    let set: [Any] = ["set", ranges, [], false, true]
    raw["code"] = [set, ["accept"]]
    XCTAssertLessThan(try readRegexV3(raw).bytes, 1_048_576)
    raw["code"] = [set, set, ["accept"]]
    XCTAssertThrowsError(try readRegexV3(raw))
    raw["code"] = Array(repeating: ["accept"], count: 4097)
    XCTAssertThrowsError(try readRegexV3(raw))
  }
}
