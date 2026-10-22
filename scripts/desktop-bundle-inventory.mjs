#!/usr/bin/env node
// A local build-change check, not artifact authentication or concurrent-build locking.
import { createHash } from "node:crypto";
import { lstatSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
const [mode, directory, manifest] = process.argv.slice(2);
if (!["snapshot", "changed"].includes(mode) || !directory || !manifest) throw Error("Usage: desktop-bundle-inventory.mjs snapshot|changed BUNDLE_DIR MANIFEST");
const installable = path => /\.(deb|rpm|AppImage|dmg|msi|exe|app\.tar\.gz)$/.test(path);
function inventory(root) {
  const entries = {};
  function walk(path) {
    let stat;
    try { stat = lstatSync(path, { bigint: true }); } catch (error) { if (error.code === "ENOENT") return; throw error; }
    if (stat.isDirectory()) { for (const name of readdirSync(path).sort()) walk(join(path, name)); }
    else if (stat.isFile() && installable(path)) entries[resolve(path)] = {
      size: String(stat.size), modified_ns: String(stat.mtimeNs), inode: String(stat.ino), device: String(stat.dev),
      sha256: createHash("sha256").update(readFileSync(path)).digest("hex"),
    };
  }
  walk(root); return entries;
}
const root = resolve(directory);
if (mode === "snapshot") writeFileSync(manifest, JSON.stringify({ version: 1, root, entries: inventory(root) }));
else {
  const before = JSON.parse(readFileSync(manifest, "utf8"));
  if (before.version !== 1 || before.root !== root || !before.entries || typeof before.entries !== "object") throw Error("Bundle snapshot does not belong to this build output");
  const changed = Object.entries(inventory(root)).filter(([path, identity]) => JSON.stringify(before.entries[path]) !== JSON.stringify(identity)).map(([path]) => path);
  const formats = new Set();
  for (const path of changed) {
    const format = path.endsWith(".app.tar.gz") ? "app.tar.gz" : path.split(".").at(-1);
    if (formats.has(format)) throw Error(`Ambiguous current bundle format ${format}; use an isolated build output`);
    formats.add(format);
  }
  for (const path of changed) process.stdout.write(`${path}\0`);
}
