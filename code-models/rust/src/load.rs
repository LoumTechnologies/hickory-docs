//! Finding every file in a Cargo workspace, and reading each one once.
//!
//! Two things decide what a Rust model can see, and neither is the directory
//! tree:
//!
//! * **`cargo metadata` decides which packages exist.** A workspace member
//!   can live anywhere a `path =` points, and a directory full of `.rs` files
//!   can belong to no crate at all. Walking `**/*.rs` reads `target/`, misses
//!   a member outside the tree, and cannot say which package a file is in —
//!   three wrong answers for one shortcut.
//! * **`mod` decides which files are part of a crate.** A `.rs` file nobody
//!   declares is not compiled, and a `#[path]` attribute puts a module
//!   somewhere the name does not. So the module tree is walked from each
//!   crate root, following declarations, and a file that is never reached is
//!   never read.
//!
//! `#[cfg]` gates are **followed and recorded**, never evaluated. This server
//! has no feature resolution and no target triple, so deciding that
//! `#[cfg(unix)] mod imp;` is out of scope would be inventing an answer; the
//! module is read and carries `cfg: ["unix"]`, and the caller — who knows
//! what it is generating for — decides.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use crate::lower::{self, Ctx};
use crate::model::{
    CrateKind, CrateRoot, Item, Module, Package, ServerInfo, Span, UnresolvedReference,
};
use crate::syntax::{attrs, span, visibility};

/// Everything the server can answer from.
pub struct Model {
    pub root: PathBuf,
    pub packages: Vec<Package>,
    pub modules: Vec<Module>,
    pub unresolved: Vec<UnresolvedReference>,
    pub file_count: usize,
}

impl Model {
    pub fn info(&self) -> ServerInfo {
        ServerInfo {
            name: "hick-model-rust".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            language: "rust".into(),
            root: self.root.display().to_string(),
            file_count: self.file_count as i32,
        }
    }

    /// Every item in the workspace, module order preserved.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.modules.iter().flat_map(|m| m.items.iter())
    }
}

/// Read a workspace.
pub fn load(root: &Path) -> Result<Model, String> {
    let manifest = manifest_for(root).ok_or_else(|| {
        format!(
            "no Cargo.toml at or above {}.\n  \
             This models a Cargo workspace, so it needs one — point it at the \
             directory holding your Cargo.toml, e.g. `hick-model-rust .`.\n  \
             A directory of loose .rs files has no packages and no crate roots, \
             and there is nothing here that could invent them.",
            root.display()
        )
    })?;

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest)
        .no_deps()
        .exec()
        .map_err(|e| {
            format!(
                "`cargo metadata` failed for {}: {e}\n  \
                 It is what says which packages exist and where each crate root \
                 is. The usual causes are a syntax error in a Cargo.toml, a \
                 `path =` pointing at a directory that is not there, and `cargo` \
                 not being on PATH.",
                manifest.display()
            )
        })?;

    let members: HashSet<_> = metadata.workspace_members.iter().collect();
    let mut packages = Vec::new();
    let mut roots = Vec::new();
    for package in &metadata.packages {
        if !members.contains(&package.id) {
            continue;
        }
        let mut crate_roots = Vec::new();
        for target in &package.targets {
            let Some(kind) = crate_kind(target) else {
                continue;
            };
            let file = PathBuf::from(target.src_path.as_std_path());
            let name = target.name.replace('-', "_");
            crate_roots.push(CrateRoot {
                name: name.clone(),
                kind,
                file: relative(root, &file),
            });
            roots.push((package.name.to_string(), name, file));
        }
        packages.push(Package {
            name: package.name.to_string(),
            version: package.version.to_string(),
            manifest_path: relative(root, package.manifest_path.as_std_path()),
            crate_roots,
        });
    }

    // Every crate name in the workspace, so a `use` can be told apart from an
    // import of somebody else's crate. Gathered before any file is read,
    // because the first file may `use` the last crate.
    let workspace_crates: Vec<String> = roots.iter().map(|(_, name, _)| name.clone()).collect();

    let mut walk = Walk {
        root: root.to_path_buf(),
        modules: Vec::new(),
        unresolved: Vec::new(),
        files: BTreeSet::new(),
        seen: HashSet::new(),
        workspace_crates,
    };
    for (package, crate_name, file) in &roots {
        let dir = file.parent().unwrap_or(root).to_path_buf();
        walk.module(file, &dir, &dir, "crate", package, crate_name, &[], None);
    }
    walk.resolve_uniform_paths();
    walk.report_external_crates();

    Ok(Model {
        root: root.to_path_buf(),
        packages,
        modules: walk.modules,
        unresolved: walk.unresolved,
        file_count: walk.files.len(),
    })
}

fn crate_kind(target: &cargo_metadata::Target) -> Option<CrateKind> {
    // A target's first kind is the one that decides how it is built; the
    // extra entries a `crate-type` list adds say how it is linked, which is
    // not a question about the source.
    target
        .kind
        .first()
        .map(|kind| match kind.to_string().as_str() {
            "lib" | "rlib" | "dylib" | "cdylib" | "staticlib" => CrateKind::Lib,
            "bin" => CrateKind::Bin,
            "proc-macro" => CrateKind::ProcMacro,
            // Integration tests, benches and examples are real crates and are
            // read: an example is often the clearest statement of an API, and
            // a generator asked about one should not be told it does not
            // exist. `kind` says which is which.
            "test" => CrateKind::Test,
            "bench" => CrateKind::Bench,
            "example" => CrateKind::Example,
            _ => CrateKind::Other,
        })
}

fn manifest_for(root: &Path) -> Option<PathBuf> {
    let mut here = root.canonicalize().ok()?;
    loop {
        let candidate = here.join("Cargo.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        here = here.parent()?.to_path_buf();
    }
}

fn relative(root: &Path, path: &Path) -> String {
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    canonical
        .strip_prefix(&canonical_root)
        .unwrap_or(&canonical)
        .display()
        .to_string()
}

struct Walk {
    root: PathBuf,
    modules: Vec<Module>,
    unresolved: Vec<UnresolvedReference>,
    /// Every file actually read, for the count the server reports.
    files: BTreeSet<PathBuf>,
    /// `<crate>\0<file>`: one file may belong to two crates through
    /// `#[path]`, and reading it once per crate is right while reading it
    /// twice for the same crate is a cycle.
    seen: HashSet<String>,
    workspace_crates: Vec<String>,
}

impl Walk {
    /// Read one file as a module, then follow its `mod` declarations.
    #[allow(clippy::too_many_arguments)]
    fn module(
        &mut self,
        file: &Path,
        dir: &Path,
        attr_dir: &Path,
        path: &str,
        package: &str,
        crate_name: &str,
        cfg: &[String],
        declared: Option<DeclaredMod>,
    ) {
        // A file reached twice — two crates sharing a module through
        // `#[path]`, which is how a workspace shares one file between a lib
        // and a bin — is read once per crate and no more, because a cycle
        // here is an infinite loop rather than a wrong answer.
        let key = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
        if !self
            .seen
            .insert(format!("{crate_name}\u{0}{}", key.display()))
        {
            return;
        }
        self.files.insert(key);

        let source = match std::fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) => {
                self.unresolved.push(UnresolvedReference {
                    message: format!(
                        "could not read {} for module `{path}`: {error}",
                        relative(&self.root, file)
                    ),
                    at: None,
                });
                return;
            }
        };
        let parsed = match syn::parse_file(&source) {
            Ok(parsed) => parsed,
            Err(error) => {
                // A file that does not parse is named, and the rest of the
                // workspace is still served. A model server that refused to
                // start because one file is mid-edit would be useless in the
                // only situation anyone runs it in.
                self.unresolved.push(UnresolvedReference {
                    message: format!("could not parse {}: {error}", relative(&self.root, file)),
                    at: Some(Span::empty(&relative(&self.root, file))),
                });
                return;
            }
        };

        let name = relative(&self.root, file);
        let inner = attrs(&name, &parsed.attrs);
        let mut cfg_here = cfg.to_vec();
        cfg_here.extend(inner.cfg.clone());

        let ctx = Ctx {
            file: &name,
            module: path,
            package,
            crate_name,
            cfg: &cfg_here,
        };
        let mut items = Vec::new();
        for node in &parsed.items {
            items.extend(lower::item(&ctx, node, &self.workspace_crates));
        }

        let declared = declared.unwrap_or_default();
        self.modules.push(Module {
            path: path.to_string(),
            package: package.to_string(),
            crate_name: crate_name.to_string(),
            file: name.clone(),
            is_inline: false,
            path_attribute: declared.path_attribute.clone(),
            visibility: declared
                .visibility
                .unwrap_or_else(crate::model::Visibility::inherited),
            attributes: declared
                .attributes
                .into_iter()
                .chain(inner.attributes.into_iter().filter(|a| !a.is_doc))
                .collect(),
            cfg: cfg_here.clone(),
            docs: inner.docs,
            items,
        });

        self.children(
            &parsed.items,
            file,
            dir,
            attr_dir,
            path,
            package,
            crate_name,
            &cfg_here,
        );
    }

    /// Follow every `mod` in a parsed file.
    #[allow(clippy::too_many_arguments)]
    fn children(
        &mut self,
        items: &[syn::Item],
        file: &Path,
        dir: &Path,
        attr_dir: &Path,
        path: &str,
        package: &str,
        crate_name: &str,
        cfg: &[String],
    ) {
        for node in items {
            let syn::Item::Mod(m) = node else { continue };
            let name = relative(&self.root, file);
            let read = attrs(&name, &m.attrs);
            let path_attribute = read
                .attributes
                .iter()
                .find(|a| a.name == "path")
                .and_then(|a| a.args.first().and_then(|v| v.value.clone()));
            let child_path = format!("{path}::{}", m.ident);
            let mut child_cfg = cfg.to_vec();
            child_cfg.extend(read.cfg.clone());
            let declared = DeclaredMod {
                path_attribute: path_attribute.clone(),
                visibility: Some(visibility(&m.vis)),
                attributes: read
                    .attributes
                    .iter()
                    .filter(|a| !a.is_doc)
                    .cloned()
                    .collect(),
            };

            match &m.content {
                // `mod foo { … }` — the same module, written in place. Its own
                // children resolve in `dir/foo`, which is the rule that makes
                // an inline module holding `mod bar;` work at all.
                Some((_, inner_items)) => {
                    let child_dir = match &path_attribute {
                        Some(p) => dir.join(p),
                        None => dir.join(m.ident.to_string()),
                    };
                    // A `#[path]` inside an inline module is relative to the
                    // chain of inline module names below the file, not to the
                    // module directory — the two differ the moment a file
                    // module has an inline module in it.
                    let child_attr_dir = match &path_attribute {
                        Some(p) => attr_dir.join(p),
                        None => attr_dir.join(m.ident.to_string()),
                    };
                    let ctx = Ctx {
                        file: &name,
                        module: &child_path,
                        package,
                        crate_name,
                        cfg: &child_cfg,
                    };
                    let mut lowered = Vec::new();
                    for item in inner_items {
                        lowered.extend(lower::item(&ctx, item, &self.workspace_crates));
                    }
                    self.modules.push(Module {
                        path: child_path.clone(),
                        package: package.to_string(),
                        crate_name: crate_name.to_string(),
                        file: name.clone(),
                        is_inline: true,
                        path_attribute: path_attribute.clone(),
                        visibility: visibility(&m.vis),
                        attributes: declared.attributes.clone(),
                        cfg: child_cfg.clone(),
                        docs: read.docs.clone(),
                        items: lowered,
                    });
                    self.children(
                        inner_items,
                        file,
                        &child_dir,
                        &child_attr_dir,
                        &child_path,
                        package,
                        crate_name,
                        &child_cfg,
                    );
                }
                None => match self.file_for(
                    dir,
                    attr_dir,
                    &m.ident.to_string(),
                    path_attribute.as_deref(),
                ) {
                    Some((child_file, child_dir)) => self.module(
                        &child_file,
                        &child_dir,
                        child_file.parent().unwrap_or(dir),
                        &child_path,
                        package,
                        crate_name,
                        &child_cfg,
                        Some(declared),
                    ),
                    None => self.unresolved.push(UnresolvedReference {
                        message: format!(
                            "`mod {}` in {} declares a module with no file: neither {}.rs nor \
                             {}/mod.rs exists under {}",
                            m.ident,
                            name,
                            m.ident,
                            m.ident,
                            relative(&self.root, dir)
                        ),
                        at: Some(span(&name, m.ident.span())),
                    }),
                },
            }
        }
    }

    /// Where `mod foo;` lives, and where its own children will.
    ///
    /// Two different base directories, which is the part that is easy to get
    /// wrong and was: a bare `mod foo;` resolves against the **module**
    /// directory (`foo.rs`'s children live in `foo/`), while `#[path = "…"]`
    /// resolves against the directory of the **file the declaration is
    /// written in**. Collapsing them reported every `#[path]`-declared test
    /// module in a real workspace as missing — 183 of them, in the first
    /// workspace this was pointed at.
    fn file_for(
        &self,
        dir: &Path,
        attr_dir: &Path,
        name: &str,
        path_attribute: Option<&str>,
    ) -> Option<(PathBuf, PathBuf)> {
        let file = match path_attribute {
            Some(relative_path) => attr_dir.join(relative_path),
            None => {
                // `foo.rs` first and `foo/mod.rs` second, the order rustc
                // itself resolves in — having both is an error rustc reports,
                // not something to guess at here.
                let flat = dir.join(format!("{name}.rs"));
                if flat.is_file() {
                    flat
                } else {
                    dir.join(name).join("mod.rs")
                }
            }
        };
        if !file.is_file() {
            return None;
        }
        // A module's own directory is its file's path without `.rs`, except
        // for `mod.rs`, which stands for the directory it is in.
        let child_dir = if file.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            file.parent().unwrap_or(dir).to_path_buf()
        } else {
            file.with_extension("")
        };
        Some((file, child_dir))
    }

    /// Bind the `use` paths that name a module rather than a crate.
    ///
    /// Rust 2018's uniform paths let `use backend::Block;` mean the sibling
    /// module `backend`, not a crate called `backend`. A resolver that only
    /// knew about crates called every one of those an external dependency —
    /// which is a lie in the one direction that matters, because `unresolved`
    /// is what a generator trusts when it decides whether it saw everything.
    ///
    /// A module in the same crate wins over a crate of the same name, which
    /// is also what rustc does with an ambiguity it can resolve locally.
    fn resolve_uniform_paths(&mut self) {
        let known: BTreeSet<String> = self
            .modules
            .iter()
            .map(|m| format!("{}\u{0}{}", m.crate_name, m.path))
            .collect();
        for module in &mut self.modules {
            let here = module.path.clone();
            let crate_name = module.crate_name.clone();
            for item in &mut module.items {
                let Item::Use(use_decl) = item else { continue };
                if use_decl.is_workspace_local {
                    continue;
                }
                let candidate = format!("{crate_name}\u{0}{here}::{}", use_decl.root);
                if known.contains(&candidate) {
                    use_decl.is_workspace_local = true;
                    use_decl.crate_name_of_target = Some(crate_name.clone());
                }
            }
        }
    }

    /// Name every crate imported from outside the workspace, once each.
    ///
    /// The rule is `code-models/README.md`'s: say what you could not resolve.
    /// A generator asking "what implements this trait" against a workspace
    /// that gets the trait from `serde` needs to know the answer stops at the
    /// workspace boundary, and one line per crate is the cheapest true way to
    /// tell it.
    fn report_external_crates(&mut self) {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut found: Vec<(String, String, Span)> = Vec::new();
        for module in &self.modules {
            for item in &module.items {
                let Item::Use(use_decl) = item else { continue };
                if use_decl.is_workspace_local {
                    continue;
                }
                let Some(crate_name) = use_decl.crate_name_of_target.clone() else {
                    continue;
                };
                if seen.insert(crate_name.clone()) {
                    found.push((
                        crate_name,
                        use_decl.path.clone(),
                        use_decl.declaration.clone(),
                    ));
                }
            }
        }
        for (crate_name, example, at) in found {
            self.unresolved.push(UnresolvedReference {
                message: format!(
                    "`{crate_name}` is outside this workspace, so paths through it are named \
                     and not bound — e.g. `use {example}`. Ask about it in the crate that \
                     defines it, or treat the name as opaque."
                ),
                at: Some(at),
            });
        }
    }
}

/// What a `mod foo;` said about the module before its file was opened.
#[derive(Default)]
struct DeclaredMod {
    path_attribute: Option<String>,
    visibility: Option<crate::model::Visibility>,
    attributes: Vec<crate::model::Attribute>,
}
