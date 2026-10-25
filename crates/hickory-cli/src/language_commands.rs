//! Reporting for built-in language support and plugin declarations.
use anyhow::Result;
use std::process::ExitCode;

pub fn plugins() -> Result<ExitCode> {
    for plugin in hick_plugins::registry().plugins() {
        let interfaces: Vec<_> = plugin
            .interfaces
            .iter()
            .map(|interface| format!("{:?}", interface.kind()))
            .collect();
        println!("{}: {}", plugin.id, interfaces.join(", "));
    }
    Ok(ExitCode::SUCCESS)
}

/// `hick lang` — the support ladder, measured on this machine.
pub fn languages() -> Result<ExitCode> {
    use hickory_cli::language_tier::{Tier, survey};

    let root = std::env::current_dir()?;
    let mut rows = survey(&root);
    // Best-supported first: the interesting rows are the ones near the top of
    // the ladder, and the long Bronze tail is context rather than news.
    rows.sort_by(|a, b| b.tier.cmp(&a.tier).then(a.language.cmp(b.language)));

    println!(
        "What Hickory knows about each language, in {}.\n",
        root.display()
    );
    println!(
        "  {:<16} {:<7} {:<5} {:<5} {:<6} {:<6} {:<6} {:<7} NEXT",
        "LANGUAGE", "TIER", "DRAW", "LSP", "DEBUG", "INDEX", "MODEL", "CLIENT"
    );
    for row in &rows {
        println!(
            "  {:<16} {:<7} {:<5} {:<5} {:<6} {:<6} {:<6} {:<7} {}",
            row.language,
            row.tier.name(),
            row.draw.mark(),
            row.lsp.mark(),
            row.dap.mark(),
            row.index.mark(),
            row.model.mark(),
            row.client.mark(),
            row.next.as_deref().unwrap_or("")
        );
    }

    let gold = rows.iter().filter(|r| r.tier == Tier::Gold).count();
    let silver = rows.iter().filter(|r| r.tier == Tier::Silver).count();
    let bronze = rows.iter().filter(|r| r.tier == Tier::Bronze).count();
    println!("\n  {gold} gold, {silver} silver, {bronze} bronze.");
    println!(
        "  `yes` is installed here; `get` is available and not installed yet.\n\
         \n\
         \x20 BRONZE  the text is right — the language is routed and runnable in a cell, and\n\
         \x20         DRAW says whether the editor has a grammar to colour it with.\n\
         \x20 SILVER  the editor is right — a language server and a debugger, in document coordinates.\n\
         \x20 GOLD    the code is data — an index, a code model to generate FROM, and a\n\
         \x20         typed-client emitter to write a generator IN."
    );
    Ok(ExitCode::SUCCESS)
}
