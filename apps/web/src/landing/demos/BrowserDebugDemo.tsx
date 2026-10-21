import { useState } from "react";
import { useUiCommands, useUiRevision } from "../../lib/uiActionBridge";
import { useUiReading } from "../../lib/uiReading";
import { DebuggableDocument } from "../../embed/DebuggableDocument";

export function browserExample(language: "ts" | "js") {
  const type = language === "ts" ? ": string" : "";
  return `# A note with code\n\n<hick:file path="hello.${language}">\nfunction greet(name${type})${type} {\n  var message = "Hello, " + name;\n  return message;\n}\nconsole.log(greet("developer"));\n</hick:file>\n`;
}
export function BrowserDebugDemo() {
  const [language, setLanguage] = useState<"ts" | "js">("ts");
  const [source, setSource] = useState(() => browserExample("ts"));
  const [revision, setRevision] = useState(0);
  const [limits, setLimits] = useState(false);
  const changeLanguage = (next: "ts" | "js") => { setLanguage(next); setSource(browserExample(next)); setRevision(r => r + 1); };
  const stateRevision = useUiRevision(() => [language, source, revision, limits]);
  useUiReading("browser debugger example", { language, limits_shown: limits, execution: "live browser; not verified cell execution", ...(limits ? { limits: "ES5 statements, functions, objects and arrays, plus TypeScript annotations. Use var. No imports, async, classes, DOM or network APIs." } : {}) });
  useUiCommands([
    { key: "browser-example:language", command: { label: "Choose browser debugger example language", parent: "browser-example", subject: "browser-example", parameters: { language: { type: "string", maxLength: 2, choices: ["ts", "js"] } }, prerequisites: ["Replaces this example's local source with its seeded language fixture."], read: () => ({ revision: stateRevision(), enabled: true }), execute: async args => { changeLanguage(args.language as "ts" | "js"); return { status: "completed", value: { language_requested: args.language } }; } } },
    { key: "browser-example:limits", command: { label: "Set browser debugger limits visibility", parent: "browser-example", subject: "browser-example:limits", parameters: { shown: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: stateRevision(), enabled: true }), execute: async args => { setLimits(args.shown as boolean); return { status: "completed", value: { visibility_requested: args.shown } }; } } },
  ]);
  const breakpoint = browserExample(language).split("\n").findIndex((line) => line.trim() === "return message;");
  return <section className="landing-demo" aria-labelledby="browser-debug-h">
    <div className="landing-demo-heading">
      <div><h2 id="browser-debug-h">Try a note. It’s already paused.</h2>
        <p>Step through the code, inspect a value, or edit and restart.</p></div>
      <label>Language <select aria-label="Debug language" value={language} onChange={(event) => {
        changeLanguage(event.target.value as "ts" | "js");
      }}><option value="ts">TypeScript</option><option value="js">JavaScript</option></select></label>
    </div>
    <div className="landing-document-name">hello.md</div>
    <DebuggableDocument key={language} path="hello.md" source={source} revision={String(revision)} startPausedAt={breakpoint}
      onChange={(change) => { setSource(change.source); setRevision((r) => r + 1); }} />
    <p className="landing-demo-note">This live JavaScript / TypeScript example runs in your browser. The full IDE is downloadable.</p>
    <details className="landing-demo-limits" open={limits} onToggle={event => setLimits(event.currentTarget.open)}><summary>Browser demo limits</summary>
      <p>ES5 statements, functions, objects and arrays, plus TypeScript annotations. Use <code>var</code>.
        No imports, async, classes, DOM or network APIs. Debugging output is not verified cell execution.</p>
    </details>
  </section>;
}
