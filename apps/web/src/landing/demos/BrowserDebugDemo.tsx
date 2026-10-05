import { useState } from "react";
import { DebuggableDocument } from "../../embed/DebuggableDocument";

export function browserExample(language: "ts" | "js") {
  const type = language === "ts" ? ": number" : "";
  return `# A calculation you can debug\n\nClick beside a code line to set a breakpoint.\n\n<hick:file path="price.${language}">\nfunction price(quantity${type})${type} {\n  var subtotal = quantity * 12;\n  var discount = subtotal > 40 ? 5 : 0;\n  return subtotal - discount;\n}\n\nvar quantity${type} = 4;\nvar total = price(quantity);\nconsole.log("Total:", total);\n</hick:file>\n`;
}
export function BrowserDebugDemo() {
  const [language, setLanguage] = useState<"ts" | "js">("ts");
  const [source, setSource] = useState(() => browserExample("ts"));
  const [revision, setRevision] = useState(0);
  return <section className="landing-demo" aria-labelledby="browser-debug-h">
    <h2 id="browser-debug-h">Edit it. Break on a line. Inspect the real values.</h2>
    <p className="landing-demo-lead">JavaScript and TypeScript run here in your browser. Set a breakpoint on <code>var total</code>,
      press Debug, then Step into. Change the quantity and restart to see a different result.</p>
    <label>Language <select aria-label="Debug language" value={language} onChange={(event) => {
      const next = event.target.value as "ts" | "js"; setLanguage(next); setSource(browserExample(next)); setRevision((r) => r + 1);
    }}><option value="ts">TypeScript</option><option value="js">JavaScript</option></select></label>
    <DebuggableDocument key={language} path="browser-demo.md" source={source} revision={String(revision)}
      onChange={(change) => { setSource(change.source); setRevision((r) => r + 1); }} />
    <p>Supported: ES5 statements, functions, closures, objects and arrays; TypeScript type annotations. Use <code>var</code>.
      No imports, async, classes, dynamic code, DOM or network APIs. Output is a live debugging run, not verified cell evidence.</p>
  </section>;
}
