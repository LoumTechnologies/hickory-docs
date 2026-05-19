---
id: TASK-36.2
title: Implement XmlSessionLog bundle writer in hick-agent-sdk
status: Done
assignee:
  - nate@loumtechnologies.com
created_date: '2026-05-19 11:00'
updated_date: '2026-05-19 11:12'
labels:
  - agent
  - logging
dependencies:
  - TASK-36.1
parent_task_id: TASK-36
priority: high
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the concrete `XmlSessionLog` struct in `hick-agent-sdk` that writes Agent Log XML to a session directory bundle.

## Depends on

TASK-36.1 — provides `SessionLog` trait, `LogEvent`, `LogAttachment`, and the XML element schema.

## Behaviour

- `XmlSessionLog::new(dir: PathBuf)` creates (or opens) a session bundle directory and opens `session.xml` inside it
- `record(&self, event: LogEvent)` serializes the event to the appropriate `<log:...>` element and appends it to `session.xml`; writes any `LogAttachment` data files to the bundle directory
- Session XML is opened with `<?xml version="1.0" encoding="UTF-8"?>` and the `<log:session>` root element written on construction; `SessionEnd` closes the root element
- Text content is XML-escaped (bare `<`, `>`, `&` in content must be escaped; the hick pipeline format does NOT apply here — this is standard XML)
- Writes are synchronous and append-only; the file is flushed after each record call so partial sessions are readable
- `LogEvent::Custom { xml_fragment, attachments }` appends the fragment verbatim (caller is responsible for well-formedness) and writes attachments to the bundle dir

## Diagnostic log integration

Implement a `log::Log` adapter (`XmlDiagnosticLogger`) that formats `log::Record` entries as `LogEvent::DiagnosticLog` and forwards them to an `Arc&lt;dyn SessionLog&gt;`. This is how `env_logger` gets replaced: binary crates call `log::set_boxed_logger(Box::new(XmlDiagnosticLogger::new(session_log.clone())))` at startup instead of `env_logger::init()`.

## Crate location

New module `hick-agent-sdk/src/xml_session_log.rs`. Export `XmlSessionLog` and `XmlDiagnosticLogger` from `hick-agent-sdk::lib`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 XmlSessionLog::new(dir: PathBuf) creates the bundle directory and session.xml, writing the XML declaration and opening log:session root element
- [x] #2 record() correctly serializes every LogEvent variant to its corresponding log: element with a time attribute
- [x] #3 Text content in all elements is XML-escaped
- [x] #4 LogEvent::Custom xml_fragment is appended verbatim; attachment data files are written to the bundle directory at the specified relative filenames
- [x] #5 session.xml is flushed after every record() call so partial sessions remain readable
- [x] #6 XmlDiagnosticLogger implements log::Log and forwards records as LogEvent::DiagnosticLog to an Arc<dyn SessionLog>
- [x] #7 Unit tests cover: each core LogEvent variant produces correct XML, text escaping, attachment file creation, and diagnostic log forwarding
<!-- AC:END -->
