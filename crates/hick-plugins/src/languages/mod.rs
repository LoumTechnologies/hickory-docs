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
mod vb;
mod xml;
mod yaml;
mod zig;
pub static PLUGINS: &[&crate::Plugin] = &[
    &rust::PLUGIN,
    &python::PLUGIN,
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
