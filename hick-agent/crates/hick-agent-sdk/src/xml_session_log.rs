//! Concrete [`SessionLog`] implementation that writes Agent Log XML to a directory bundle.

use std::io::{BufWriter, Write as _};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::Local;

use crate::session_log::{LogEvent, LogLevel, SessionLog};

// ---------------------------------------------------------------------------
// XML helpers
// ---------------------------------------------------------------------------

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn now_rfc3339() -> String {
    Local::now().to_rfc3339()
}

// ---------------------------------------------------------------------------
// XmlSessionLog
// ---------------------------------------------------------------------------

struct Inner {
    writer: BufWriter<std::fs::File>,
    bundle_dir: PathBuf,
}

/// Writes Agent Log XML to a session directory bundle.
///
/// On construction the bundle directory is created (or opened) and `session.xml`
/// is opened for append-only writing.  The XML declaration and `<log:session>`
/// root element are written immediately.
///
/// Each [`SessionLog::record`] call serializes the event to the appropriate
/// `<log:...>` element and flushes the file so partial sessions remain readable.
///
/// `LogEvent::SessionEnd` writes the closing `</log:session>` tag.
/// `LogEvent::Custom` appends the caller-supplied XML fragment verbatim and
/// writes any [`LogAttachment`][crate::LogAttachment] data files into the bundle directory.
pub struct XmlSessionLog {
    inner: Mutex<Inner>,
}

impl XmlSessionLog {
    /// Create (or open) a session bundle at `dir` and begin writing `session.xml`.
    ///
    /// Returns an error if the directory cannot be created or the file cannot be opened.
    pub fn new(dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&dir)?;
        let xml_path = dir.join("session.xml");
        let file = std::fs::File::create(&xml_path)?;
        let mut writer = BufWriter::new(file);

        let start = now_rfc3339();
        writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            writer,
            r#"<log:session xmlns:log="http://www.hickorydocs.com/agent-log/1.0" start="{start}">"#
        )?;
        writer.flush()?;

        Ok(Self {
            inner: Mutex::new(Inner { writer, bundle_dir: dir }),
        })
    }
}

impl SessionLog for XmlSessionLog {
    fn record(&self, event: LogEvent<'_>) {
        let Ok(mut guard) = self.inner.lock() else { return };
        let Inner { writer, bundle_dir } = &mut *guard;
        let time = now_rfc3339();

        let result: std::io::Result<()> = (|| {
            match event {
                LogEvent::SessionStart { model, sandbox_image, max_turns } => {
                    let sandbox_attr = sandbox_image
                        .map(|s| format!(r#" sandbox="{}""#, xml_escape(s)))
                        .unwrap_or_default();
                    writeln!(
                        writer,
                        r#"  <log:session-start model="{}" max-turns="{}"{} time="{}"/>"#,
                        xml_escape(model),
                        max_turns,
                        sandbox_attr,
                        time,
                    )?;
                }

                LogEvent::UserTurn { text } => {
                    writeln!(
                        writer,
                        r#"  <log:user time="{}">{}</log:user>"#,
                        time,
                        xml_escape(text),
                    )?;
                }

                LogEvent::StateTransition { from, to } => {
                    writeln!(
                        writer,
                        r#"  <log:state-transition from="{}" to="{}" time="{}"/>"#,
                        xml_escape(from),
                        xml_escape(to),
                        time,
                    )?;
                }

                LogEvent::LlmResponse { text } => {
                    writeln!(
                        writer,
                        r#"  <log:assistant time="{}">{}</log:assistant>"#,
                        time,
                        xml_escape(text),
                    )?;
                }

                LogEvent::ActionStart { lang, code } => {
                    writeln!(
                        writer,
                        r#"  <log:action lang="{}" time="{}">{}</log:action>"#,
                        xml_escape(lang),
                        time,
                        xml_escape(code),
                    )?;
                }

                LogEvent::ActionEnd { stdout, stderr, exit_code } => {
                    let exit_attr = exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".into());
                    let combined = if stderr.is_empty() {
                        stdout.to_string()
                    } else if stdout.is_empty() {
                        stderr.to_string()
                    } else {
                        format!("{stdout}{stderr}")
                    };
                    writeln!(
                        writer,
                        r#"  <log:observation exit="{}" time="{}">{}</log:observation>"#,
                        exit_attr,
                        time,
                        xml_escape(&combined),
                    )?;
                }

                LogEvent::McpCall { server, method, params } => {
                    let json = serde_json::to_string(params).unwrap_or_default();
                    writeln!(
                        writer,
                        r#"  <log:mcp-call server="{}" method="{}" time="{}"><![CDATA[{}]]></log:mcp-call>"#,
                        xml_escape(server),
                        xml_escape(method),
                        time,
                        json,
                    )?;
                }

                LogEvent::McpResponse { server, method, result } => {
                    let json = serde_json::to_string(result).unwrap_or_default();
                    writeln!(
                        writer,
                        r#"  <log:mcp-response server="{}" method="{}" time="{}"><![CDATA[{}]]></log:mcp-response>"#,
                        xml_escape(server),
                        xml_escape(method),
                        time,
                        json,
                    )?;
                }

                LogEvent::SecurityViolation { description } => {
                    writeln!(
                        writer,
                        r#"  <log:security-violation time="{}">{}</log:security-violation>"#,
                        time,
                        xml_escape(description),
                    )?;
                }

                LogEvent::DiagnosticLog { level, target, message } => {
                    writeln!(
                        writer,
                        r#"  <log:diagnostic level="{}" target="{}" time="{}">{}</log:diagnostic>"#,
                        level.as_str(),
                        xml_escape(target),
                        time,
                        xml_escape(message),
                    )?;
                }

                LogEvent::SessionEnd => {
                    writeln!(
                        writer,
                        r#"  <log:session-end time="{}"/>"#,
                        time,
                    )?;
                    writeln!(writer, "</log:session>")?;
                }

                LogEvent::Custom { xml_fragment, attachments } => {
                    writer.write_all(xml_fragment.as_bytes())?;
                    if !xml_fragment.ends_with('\n') {
                        writer.write_all(b"\n")?;
                    }
                    for att in attachments {
                        let path = bundle_dir.join(&att.filename);
                        // Best-effort: ignore individual attachment errors so the
                        // session XML itself is not disrupted.
                        let _ = std::fs::write(&path, &*att.data);
                    }
                }
            }

            writer.flush()
        })();

        // Swallow write errors; session logging must not crash the agent.
        let _ = result;
    }
}

// ---------------------------------------------------------------------------
// XmlDiagnosticLogger
// ---------------------------------------------------------------------------

/// A [`log::Log`] adapter that forwards `log::Record`s to a [`SessionLog`] as
/// [`LogEvent::DiagnosticLog`] events.
///
/// Install this at binary startup to replace `env_logger`:
///
/// ```rust,ignore
/// let session_log: Arc<dyn SessionLog> = ...;
/// log::set_boxed_logger(Box::new(XmlDiagnosticLogger::new(session_log)))
///     .expect("logger already set");
/// log::set_max_level(log::LevelFilter::Info);
/// ```
pub struct XmlDiagnosticLogger {
    sink: Arc<dyn SessionLog>,
}

impl XmlDiagnosticLogger {
    pub fn new(sink: Arc<dyn SessionLog>) -> Self {
        Self { sink }
    }
}

impl log::Log for XmlDiagnosticLogger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        let level = match record.level() {
            log::Level::Trace => LogLevel::Trace,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Info => LogLevel::Info,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Error => LogLevel::Error,
        };
        let message = record.args().to_string();
        self.sink.record(LogEvent::DiagnosticLog {
            level,
            target: record.target(),
            message: &message,
        });
    }

    fn flush(&self) {}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use log::Log as _;
    use tempfile::tempdir;
    use crate::session_log::NullSessionLog;

    fn read_session_xml(dir: &std::path::Path) -> String {
        std::fs::read_to_string(dir.join("session.xml")).unwrap()
    }

    #[test]
    fn test_session_start_end() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        log.record(LogEvent::SessionStart {
            model: "claude-sonnet",
            sandbox_image: Some("alpine"),
            max_turns: 50,
        });
        log.record(LogEvent::SessionEnd);

        let xml = read_session_xml(dir.path());
        assert!(xml.contains("log:session-start"));
        assert!(xml.contains(r#"model="claude-sonnet""#));
        assert!(xml.contains(r#"sandbox="alpine""#));
        assert!(xml.contains(r#"max-turns="50""#));
        assert!(xml.contains("log:session-end"));
        assert!(xml.contains("</log:session>"));
    }

    #[test]
    fn test_user_turn_and_response() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        log.record(LogEvent::UserTurn { text: "hello" });
        log.record(LogEvent::LlmResponse { text: "world" });

        let xml = read_session_xml(dir.path());
        assert!(xml.contains("<log:user"));
        assert!(xml.contains("hello"));
        assert!(xml.contains("<log:assistant"));
        assert!(xml.contains("world"));
    }

    #[test]
    fn test_xml_escaping() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        log.record(LogEvent::UserTurn { text: "a < b && c > d \"quote\"" });

        let xml = read_session_xml(dir.path());
        assert!(xml.contains("a &lt; b &amp;&amp; c &gt; d &quot;quote&quot;"));
    }

    #[test]
    fn test_state_transition() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        log.record(LogEvent::StateTransition { from: "Idle", to: "Thinking { messages_sent: 2 }" });

        let xml = read_session_xml(dir.path());
        assert!(xml.contains(r#"from="Idle""#));
        assert!(xml.contains("Thinking"));
    }

    #[test]
    fn test_action_start_end() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        log.record(LogEvent::ActionStart { lang: "python", code: "print('hi')" });
        log.record(LogEvent::ActionEnd { stdout: "hi\n", stderr: "", exit_code: Some(0) });

        let xml = read_session_xml(dir.path());
        assert!(xml.contains(r#"lang="python""#));
        assert!(xml.contains("print"));
        assert!(xml.contains(r#"exit="0""#));
        assert!(xml.contains("hi"));
    }

    #[test]
    fn test_mcp_call_response() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        let params = serde_json::json!({"tool": "read_file"});
        let result = serde_json::json!({"content": "file content"});
        log.record(LogEvent::McpCall { server: "github", method: "tools/call", params: &params });
        log.record(LogEvent::McpResponse { server: "github", method: "tools/call", result: &result });

        let xml = read_session_xml(dir.path());
        assert!(xml.contains("log:mcp-call"));
        assert!(xml.contains("log:mcp-response"));
        assert!(xml.contains("github"));
        assert!(xml.contains("read_file"));
        assert!(xml.contains("file content"));
    }

    #[test]
    fn test_security_violation() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        log.record(LogEvent::SecurityViolation { description: "policy denied" });

        let xml = read_session_xml(dir.path());
        assert!(xml.contains("log:security-violation"));
        assert!(xml.contains("policy denied"));
    }

    #[test]
    fn test_diagnostic_log() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        log.record(LogEvent::DiagnosticLog {
            level: LogLevel::Warn,
            target: "hick_agent_sdk",
            message: "something happened",
        });

        let xml = read_session_xml(dir.path());
        assert!(xml.contains(r#"level="warn""#));
        assert!(xml.contains("hick_agent_sdk"));
        assert!(xml.contains("something happened"));
    }

    #[test]
    fn test_attachment_file_creation() {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        let data: Arc<[u8]> = Arc::from(b"binary content".as_slice());
        let attachments = [crate::LogAttachment { filename: "out.bin".into(), data }];
        log.record(LogEvent::Custom {
            xml_fragment: "  <log:custom/>",
            attachments: &attachments,
        });

        let xml = read_session_xml(dir.path());
        assert!(xml.contains("<log:custom/>"));
        let att_path = dir.path().join("out.bin");
        assert!(att_path.exists());
        assert_eq!(std::fs::read(&att_path).unwrap(), b"binary content");
    }

    #[test]
    fn test_diagnostic_logger_forwarding() {
        use std::sync::Mutex;

        struct SpyLog(Mutex<Vec<String>>);
        impl SessionLog for SpyLog {
            fn record(&self, event: LogEvent<'_>) {
                if let LogEvent::DiagnosticLog { message, .. } = event {
                    self.0.lock().unwrap().push(message.to_string());
                }
            }
        }

        let spy = Arc::new(SpyLog(Mutex::new(Vec::new())));
        let logger = XmlDiagnosticLogger::new(spy.clone() as Arc<dyn SessionLog>);

        let record = log::Record::builder()
            .args(format_args!("test message"))
            .level(log::Level::Info)
            .target("test_target")
            .build();
        logger.log(&record);

        let messages = spy.0.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "test message");
    }

    #[test]
    fn test_null_session_log() {
        let log = NullSessionLog;
        // Must not panic.
        log.record(LogEvent::SessionStart { model: "m", sandbox_image: None, max_turns: 1 });
        log.record(LogEvent::SessionEnd);
    }
}
