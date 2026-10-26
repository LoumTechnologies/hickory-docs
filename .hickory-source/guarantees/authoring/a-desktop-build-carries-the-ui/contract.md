<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Desktop Build Carries The UI

For: Authors editing executable documents and the files those documents produce.

Given `just local-install` or a Tauri desktop build, when the frontend is built
and the desktop binary is compiled, then the binary embeds that frontend's
index, scripts, and styles. Rebuilding the frontend invalidates Cargo's cached
desktop build even when no Rust source changes. A release build without a built
frontend fails before it can replace the installed app.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-desktop-build-carries-the-ui.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-2c3a5e03f675"></a>
## `apps/desktop/src-tauri/build.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

Related contracts: [A Desktop Build Carries The UI](contract.md).

<a id="f-2c3a5e03f675-main"></a>
### `main`

<storycode:copy id="f-2c3a5e03f675-main">
fn main() {
    // rust-embed discovers assets during macro expansion. Cargo must also
    // watch the directory: rebuilding the UI can add/remove hashed filenames
    // without changing any Rust source (including after an empty dist build).
    println!("cargo:rerun-if-changed=../../web/dist");
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        assert!(
            std::path::Path::new("../../web/dist/index.html").is_file(),
            "The desktop release requires the built UI. Run `just build-web` or build through `cargo tauri build`."
        );
    }
    tauri_build::build()
}
</storycode:copy>

### Assemble `apps/desktop/src-tauri/build.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/build.rs"><storycode:paste select="#f-2c3a5e03f675-main"/></storycode:file>

## Test the behavior: `the_page_serves_its_built_scripts_and_styles`

This is the maintained test fragment from `apps/desktop/src-tauri/tests/serves_one_origin.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e7ec95ab75f6-the_page_serves_its_built_scripts_and_styles"></a>
### `the_page_serves_its_built_scripts_and_styles`

<storycode:copy id="f-e7ec95ab75f6-the_page_serves_its_built_scripts_and_styles">
/// Guarantee: docs/guarantees/authoring/a-desktop-build-carries-the-ui.md
#[tokio::test(flavor = "multi_thread")]
async fn the_page_serves_its_built_scripts_and_styles() {
    let session = hickory_desktop_lib::server::start_blank().await.unwrap();
    let http = reqwest::Client::new();
    let page = http
        .get(&session.url)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .text()
        .await
        .unwrap();
    let assets: Vec<_> = page
        .split('"')
        .filter(|value| value.starts_with("/assets/"))
        .filter(|value| value.ends_with(".js") || value.ends_with(".css"))
        .collect();
    assert!(assets.iter().any(|asset| asset.ends_with(".js")));
    assert!(assets.iter().any(|asset| asset.ends_with(".css")));
    for asset in assets {
        let response = http
            .get(format!("{}{asset}", session.url))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
        let body = response.text().await.unwrap();
        assert!(!body.is_empty(), "empty asset: {asset}");
        assert!(
            !body.contains("<div id=\"root\">"),
            "shell fallback: {asset}"
        );
    }
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/build.rs](contract.md#output-2c3a5e03f675)

### Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../mechanisms/apps/desktop/src-tauri/tests/serves_one_origin.rs.md#output-e7ec95ab75f6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
