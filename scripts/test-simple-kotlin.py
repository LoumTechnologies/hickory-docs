"""Focused Kotlin pane conformance, independent of Gradle and the Android SDK."""
import argparse
import json
from pathlib import Path
import os
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def literal(value):
    return json.dumps(value, ensure_ascii=False).replace("$", "\\$")

def run(java, compiler_home):
    compiler = Path(compiler_home) / "lib"
    stdlib = compiler / "kotlin-stdlib.jar"
    fixture = json.loads((ROOT / "apps/simple-contract/layout-cases.json").read_text())
    scenarios = fixture["cases"]
    lines = ["import com.hickorydocs.simple.PaneLayout", "import com.hickorydocs.simple.dropEdge",
        "fun axes(layout: PaneLayout, id: String = layout.rootID): List<String> { val node = layout.nodes.getValue(id); return if (node.axis == null) emptyList() else listOf(node.axis) + axes(layout, node.first!!) + axes(layout, node.second!!) }", "fun main() {"]
    for point in fixture["edgeCases"]:
        expected = literal(point["edge"]) if point["edge"] else "null"
        lines.append(f'check(dropEdge({float(point["x"])}, {float(point["y"])}) == {expected})')
    for scenario in scenarios:
        lines += ["run {", "var layout = PaneLayout()"]
        for step in scenario["steps"]:
            group = f'layout.groups()[{step["group"]}].id' if "group" in step else '""'
            if "target" in step: group = literal(step["target"])
            tab = literal(step.get("tab", ""))
            expression = {
                "open": f"activate({tab}, {group})", "move": f"move({tab}, {group}, {literal(step['before']) if 'before' in step else 'null'})", "remove": f"remove({tab})",
                "dock": f'dock({tab}, {group}, {literal(step.get("edge", ""))}, {literal(step.get("id", ""))})',
                "split": f'split({group}, {literal(step.get("axis", ""))}, {literal(step.get("id", ""))})'
            }[step["op"]]
            lines.append("layout = layout." + expression)
        expected = "listOf(" + ",".join("listOf<String>(" + ",".join(map(literal, group)) + ")" for group in scenario["expected"]) + ")"
        lines.append(f'check(layout.groups().map {{ it.tabs }} == {expected}) {{ {literal(scenario["name"])} }}')
        if "axes" in scenario:
            expected_axes = "listOf<String>(" + ",".join(map(literal, scenario["axes"])) + ")"
            lines.append(f'check(axes(layout) == {expected_axes})')
        lines += ['check(layout.groups().all { it.active == null || it.active in it.tabs })', "}"]
    lines += [f'println("Kotlin: {len(scenarios)} shared pane scenarios passed")', "}"]
    with tempfile.TemporaryDirectory(prefix="hickory-kotlin-") as directory:
        work = Path(directory); source = work / "SimpleProbe.kt"; source.write_text("\n".join(lines))
        output = work / "classes"
        layout = ROOT / "apps/android-simple/app/src/main/java/com/hickorydocs/simple/PaneLayout.kt"
        subprocess.run([java, "-cp", str(compiler / "*"), "org.jetbrains.kotlin.cli.jvm.K2JVMCompiler", "-no-reflect", "-no-stdlib", "-classpath", str(stdlib), "-jvm-target", "17", "-d", str(output), str(layout), str(source)], check=True)
        subprocess.run([java, "-cp", os.pathsep.join([str(output), str(stdlib)]), "SimpleProbeKt"], check=True)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java", required=True)
    parser.add_argument("--compiler-home", required=True)
    args = parser.parse_args()
    run(args.java, args.compiler_home)
