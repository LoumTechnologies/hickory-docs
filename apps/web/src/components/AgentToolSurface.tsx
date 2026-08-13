// The page's primary claim, shown rather than asserted: the tool surface your
// own coding agent drives.
//
// Every command, flag, file and tool name here is real and must stay real —
// they come from `crates/hickory-cli/src/main.rs` (the `doc` and `mcp`
// subcommands) and `crates/hickory-cli/src/init.rs` (which writes the
// AGENTS.md section, the `.mcp.json` entry and the pre-commit hook). A
// marketing page that invents a flag is worse than one that says nothing,
// because the visitor finds out by typing it.
//
// This section is static on purpose. The interactive demo below it shows the
// mechanism; this shows the surface, and a fake terminal that pretends to run
// an agent would only obscure how small the real surface is.

/** The five document tools, exactly as `hick doc --help` lists them. */
const TOOLS: { call: string; does: string }[] = [
  {
    call: "hick doc read <doc>",
    does: "The document source, every line prefixed with its 4-hex content hash. Those hashes are the anchors an edit takes.",
  },
  {
    call: "hick doc read-output <doc> --path f --lineage",
    does: "A generated file, plus which document span produced each range of it and whether that range can be edited from here.",
  },
  {
    call: "hick doc edit-output <doc> --path f --run aa12..bb34",
    does: "Edit the generated file. The change is mapped back into the document byte-exactly. This is how code should be edited.",
  },
  {
    call: "hick doc edit <doc> --run aa12",
    does: "Edit the document itself. This is how structure and prose should be edited — and where a refused output edit routes you.",
  },
  {
    call: "hick doc verify <doc>",
    does: "Execute the document for real — every exec cell, every expectation — and write its outputs. Run before calling an edit done.",
  },
];

export function AgentToolSurface() {
  return (
    <div className="agent-surface">
      <div className="agent-surface-step">
        <h3>1. Point it at your repository</h3>
        <pre className="agent-surface-code">
          <code>{"$ hick init"}</code>
        </pre>
        <p>
          One idempotent command. It writes an <span className="mono">AGENTS.md</span> section
          describing the grammar and the rules that keep an agent out of the generated files,
          registers <span className="mono">hick mcp</span> in the repository&rsquo;s{" "}
          <span className="mono">.mcp.json</span>, and installs a pre-commit hook that fails the
          commit when a document and reality have drifted apart. Re-run it any time; it refreshes
          the blocks it manages and touches nothing else.
        </p>
      </div>

      <div className="agent-surface-step">
        <h3>2. Your agent already speaks it</h3>
        <p>
          If your harness speaks MCP — Claude Code, Codex, Grok CLI, anything else —{" "}
          <span className="mono">hick mcp</span> serves the tools over stdio and keeps the edit
          session open between calls. If it does not, the same five tools are subcommands, so any
          agent that can run a command can drive them.
        </p>
        <dl className="agent-tools">
          {TOOLS.map((tool) => (
            <div key={tool.call} className="agent-tool">
              <dt className="mono">{tool.call}</dt>
              <dd>{tool.does}</dd>
            </div>
          ))}
        </dl>
      </div>

      <div className="agent-surface-step">
        <h3>3. The part that makes it worth doing</h3>
        <p>
          Anchors are <strong>content hashes</strong>, not line numbers. An edit written against a
          line that has changed since the agent read it is <em>refused</em> rather than applied
          somewhere it does not belong — so the failure mode where a patch lands two functions away
          from where it was written cannot happen. A refusal is routing: it names the document
          location to use instead.
        </p>
        <p>
          Set <span className="mono">HICKORY_SESSION=sessions/name.hick</span> and every tool call
          your agent makes is appended to a replayable document in your repository — what it read,
          what it changed, what the run printed. It lands as a file you review in a diff, not a
          chat log you take on faith. <span className="mono">hick promote</span> compacts a messy
          session into a clean pipeline that reproduces the same result without the dead ends.
        </p>
      </div>

      <p className="muted agent-surface-note">
        Hickory ships its own agent too (<span className="mono">hick agent</span>), running on your
        own API key from your own environment. It is the same five tools. Bringing your own is the
        supported path, not the fallback.
      </p>
    </div>
  );
}
