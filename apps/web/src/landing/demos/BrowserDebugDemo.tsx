import { useState } from "react";
import { DebuggableDocument } from "../../embed/DebuggableDocument";

export function browserExample(language: "ts" | "js") {
  const type = language === "ts" ? ": string" : "";
  return `# A note with code\n\n<hick:file path="hello.${language}">\nfunction greet(name${type})${type} {\n  var message = "Hello, " + name;\n  return message;\n}\nconsole.log(greet("developer"));\n</hick:file>\n`;
}
export function BrowserDebugDemo() {
  const [language, setLanguage] = useState<"ts" | "js">("ts");
  const [source, setSource] = useState(() => browserExample("ts"));
  const [revision, setRevision] = useState(0);
  const breakpoint = browserExample(language).split("\n").findIndex((line) => line.trim() === "return message;");
  return <section className="landing-demo" aria-labelledby="browser-debug-h">
    <div className="landing-demo-heading">
      <div><h2 id="browser-debug-h">Try a note. It’s already paused.</h2>
        <p>Step through the code, inspect a value, or edit and restart.</p></div>
      <label>Language <select aria-label="Debug language" value={language} onChange={(event) => {
        const next = event.target.value as "ts" | "js"; setLanguage(next); setSource(browserExample(next)); setRevision((r) => r + 1);
      }}><option value="ts">TypeScript</option><option value="js">JavaScript</option></select></label>
    </div>
    <div className="landing-document-name">hello.md</div>
    <DebuggableDocument key={language} path="hello.md" source={source} revision={String(revision)} startPausedAt={breakpoint}
      onChange={(change) => { setSource(change.source); setRevision((r) => r + 1); }} />
    <p className="landing-demo-note">This live JavaScript / TypeScript example runs in your browser. The full IDE is downloadable.</p>
    <details className="landing-demo-limits"><summary>Browser demo limits</summary>
      <p>ES5 statements, functions, objects and arrays, plus TypeScript annotations. Use <code>var</code>.
        No imports, async, classes, DOM or network APIs. Debugging output is not verified cell execution.</p>
    </details>
  </section>;
}
