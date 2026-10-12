// Regenerate preview.html from spatial-navigation.html.
//
// The host around the prototype (the Codex visualization iframe, its theme
// variables, tooltip/floating-ui/lucide bootstrapping and the widget-state
// bridge) is fixed and lives in preview.template.html as an escaped srcdoc
// with one placeholder. This script splices the editable prototype into it,
// so the two files can never drift the way they used to when both were
// hand-edited together.
//
//   node work/navigation-prototype/build.cjs
//
// The template is itself generated once from the tool's own output; only
// regenerate it by extracting a fresh wrapper, never by hand.

const fs = require("node:fs");
const path = require("node:path");

const dir = __dirname;
const placeholder = "__SPATIAL_NAVIGATION__";

const escapeAttribute = (value) =>
  value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#x27;");

const template = fs.readFileSync(path.join(dir, "preview.template.html"), "utf8");
const spatial = fs.readFileSync(path.join(dir, "spatial-navigation.html"), "utf8");

if (template.split(placeholder).length !== 2) {
  throw new Error(`expected exactly one ${placeholder} in preview.template.html`);
}

const output = template.replace(placeholder, escapeAttribute(spatial));
fs.writeFileSync(path.join(dir, "preview.html"), output);
console.log(`preview.html written (${spatial.length} prototype bytes)`);
