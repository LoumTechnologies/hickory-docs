mod c;
mod cpp;
mod css;
mod dap;
mod dap_install;
mod dart;
mod dockerfile;
mod elixir;
mod fsharp;
mod go;
mod groovy;
mod haskell;
mod html;
mod index_install;
mod java;
mod json;
mod kotlin;
mod lsp;
mod lsp_install;
mod lua;
mod markdown;
mod nix;
mod php;
mod python;
mod r;
mod react;
mod ruby;
mod rust;
mod scala;
mod shellscript;
mod sql;
mod swift;
mod terraform;
mod toml;
mod typescript;
pub use typescript::relative_ecmascript_imports;
mod vb;
mod xml;
mod yaml;
mod zig;
static LATEX_FORMULA: crate::Plugin = crate::Plugin {
    id: "latex",
    languages: &["latex"],
    interfaces: &[crate::Interface::Formula(crate::FormulaBackend {
        language: "latex",
        aliases: &["latex", "tex"],
        file_name: "latex_backend.py",
        interpreters: &["python3", "python"],
        source: concat!("FORMULA_LANGUAGE = 'latex'\n", include_str!("python_backend.py")),
    })],
};
pub static PLUGINS: &[&crate::Plugin] = &[
    &rust::PLUGIN,
    &python::PLUGIN,
    &LATEX_FORMULA,
    &typescript::PLUGIN,
    &react::PLUGIN,
    &go::PLUGIN,
    &c::PLUGIN,
    &cpp::PLUGIN,
    &java::PLUGIN,
    &ruby::PLUGIN,
    &php::PLUGIN,
    &kotlin::PLUGIN,
    &swift::PLUGIN,
    &scala::PLUGIN,
    &elixir::PLUGIN,
    &haskell::PLUGIN,
    &lua::PLUGIN,
    &zig::PLUGIN,
    &nix::PLUGIN,
    &json::PLUGIN,
    &yaml::PLUGIN,
    &toml::PLUGIN,
    &html::PLUGIN,
    &css::PLUGIN,
    &markdown::PLUGIN,
    &sql::PLUGIN,
    &terraform::PLUGIN,
    &dockerfile::PLUGIN,
    &shellscript::PLUGIN,
    &r::PLUGIN,
    &dart::PLUGIN,
    &xml::PLUGIN,
    &groovy::PLUGIN,
    &fsharp::PLUGIN,
    &vb::PLUGIN,
    &crate::csharp::PLUGIN,
];
