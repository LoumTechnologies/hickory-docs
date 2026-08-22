//! A cell with an id is quotable: `<hick:paste select="#cell"/>` pastes what
//! the cell shows, with the cell's exec provenance — so a finding can carry
//! the number a cell printed instead of retyping it. And a copy inside a
//! claim still registers, so a claim can wrap the fragment it asserts.
//! Guarantee: docs/guarantees/authoring/a-cell-with-an-id-is-quotable.md

fn hick() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
}

#[test]
fn a_cells_output_pastes_by_id_and_a_copy_inside_a_claim_registers() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("n.hick");
    std::fs::write(
        &doc,
        r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="n.md">
<hick:container name="sh" image="host" />

Before the cell: <hick:paste select="#p95" />

<hick:exec id="p95" container="sh" show="output">printf '212 ms'</hick:exec>

<hick:claim by="nate" standing="judgment"><hick:copy id="j" class="message">Fine for now.</hick:copy></hick:claim>

<hick:file path="out.txt">p95 is <hick:paste select="#p95" />; <hick:paste select="#j" /></hick:file>
</hick:doc>
"##,
    )
    .unwrap();
    let out = hick()
        .env("HICKORY_EXECUTOR", "local")
        .args(["run", doc.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let file = std::fs::read_to_string(dir.path().join("out.txt")).unwrap();
    assert_eq!(file.trim(), "p95 is 212 ms; Fine for now.");
    let woven = std::fs::read_to_string(dir.path().join("n.md")).unwrap();
    assert!(
        woven.contains("Before the cell: 212 ms"),
        "a paste before the cell resolves too: {woven}"
    );

    // The pasted bytes carry the cell's provenance, not a literal span.
    let lineage = hick()
        .env("HICKORY_EXECUTOR", "local")
        .args([
            "lineage",
            doc.to_str().unwrap(),
            "--output",
            "out.txt",
            "--json",
        ])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&lineage.stdout).unwrap();
    let kinds: Vec<String> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["origin"]["kind"].as_str().unwrap_or("?").to_string())
        .collect();
    assert!(kinds.iter().any(|k| k == "exec"), "{kinds:?}");
}
