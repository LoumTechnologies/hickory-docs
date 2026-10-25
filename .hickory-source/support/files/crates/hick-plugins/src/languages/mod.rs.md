<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/mod.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `aa9fad2c3f7dc5c30c4fa3ad5c9854f010d62d007fef22e6402c378622eb5b11`.

<book:entry path="crates/hick-plugins/src/languages/mod.rs">
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
</book:entry>

</book:doc>
