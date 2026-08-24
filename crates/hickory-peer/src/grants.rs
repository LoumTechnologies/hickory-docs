//! What a peer may ask this machine to do, per verb.
//!
//! See `docs/specs/freeform/one-engineer-many-machines.md`. The peer channel
//! carries **the existing API and the existing rooms** — so the gate is on
//! the request, and it is the whole trust model:
//!
//! | Grant | Lets a peer | Default |
//! |---|---|---|
//! | `view` | see documents, outputs, ribbons, transcripts, a running terminal's bytes | on |
//! | `edit` | write into the room, and thereby into the file | on |
//! | `execute` | run a cell or the up-loop, type into a shell, start or steer an agent turn | **off** |
//!
//! **Deny by default.** A path this table does not know is refused, never
//! passed through. The alternative — allow unless matched — means every route
//! added later is silently reachable by every peer, which is the failure mode
//! where a security boundary quietly stops being one. The same posture the
//! broker's host policy takes, for the same reason.
//!
//! The dangerous sentence has not gone away, it has changed owner: *a remote
//! machine can cause code to run here*, and the remote machine is yours. So
//! what this bounds is blast radius when one of your own machines is
//! compromised — it does not defeat it, and nothing here should be read as
//! claiming otherwise.

use std::collections::BTreeSet;

use hickory_fleet::Grant;

/// Why a request was refused, in the words the peer is given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denial {
    /// The grant that would have permitted it, when one would.
    pub needs: Option<Grant>,
    pub message: String,
}

impl Denial {
    fn needs(grant: Grant, method: &str, path: &str, machine: &str) -> Self {
        Self {
            needs: Some(grant),
            message: format!(
                "this machine refused `{method} {path}` from \"{machine}\": it \
                 needs the `{}` grant, which that machine does not have.\n  \
                 Grants are per machine, per verb, and `execute` is off by \
                 default because \"my laptop was stolen\" should not read as \
                 \"every machine I own now executes whatever the thief \
                 types\".\n  \
                 Next step, ON THE MACHINE YOU ARE ATTACHING TO — this text \
                 was written there and is being read here, so \"this machine\" \
                 would be ambiguous: `hick fleet grant {machine} {}`.",
                grant.as_str(),
                grant.as_str(),
            ),
        }
    }

    fn unknown(method: &str, path: &str) -> Self {
        Self {
            needs: None,
            message: format!(
                "this machine refused `{method} {path}`: the peer channel \
                 allows only routes it knows, and this is not one of them.\n  \
                 That is deliberate — allowing anything unmatched would make \
                 every route added later reachable by every paired machine \
                 without anybody deciding so."
            ),
        }
    }
}

/// The verb a request needs, or `None` when no verb covers it.
///
/// Ordered most-specific first: `execute` routes are matched before the
/// `edit` and `view` shapes they sit inside, because a cell run is a POST to
/// a document and would otherwise read as an ordinary write.
fn required(method: &str, path: &str) -> Option<Grant> {
    let method = method.to_ascii_uppercase();
    // Query strings are not part of the decision, and a peer must not be able
    // to change the verdict by appending one.
    let path = path.split('?').next().unwrap_or(path);

    // --- never, whatever the grants ---------------------------------------
    // Settings hold provider keys and the continuity switch. Changing the
    // FLEET is worse than either: a peer that could do it would grant itself
    // `execute`, or enrol a fourth machine, from inside the very channel
    // those grants are supposed to bound. Reading the roster is fine and is
    // allowed below; every other verb and every sub-path is not.
    //
    // Checked FIRST, so no prefix rule further down can accidentally re-open
    // one of them — which is exactly what `/api/fleet/invite` did when
    // `/api/fleet` was only a readable prefix.
    if path.starts_with("/api/settings")
        || (path.starts_with("/api/fleet/")
            || (path == "/api/fleet" && !matches!(method.as_str(), "GET" | "HEAD")))
    {
        return None;
    }

    // --- execute: code runs on this machine, as this user -----------------
    // A cell, the pipeline, a shell, an agent turn, a debugger, a formula
    // backend. All of them mean the same thing, which is why they are one
    // grant rather than the three this design started with.
    let executes = path.ends_with("/run")
        || path.ends_with("/check")
        || path.ends_with("/agent")
        || path.starts_with("/api/terminals")
        || path.starts_with("/api/debug")
        || path.starts_with("/api/formula/evaluate")
        || path.starts_with("/api/formula/trace")
        || path.starts_with("/api/adopt")
        || path.starts_with("/api/merged/write")
        || path.starts_with("/api/merged/undo");
    if executes {
        // Even reading these is not a read: `GET /api/terminals` lists
        // sessions, but the route family is the boundary and splitting it by
        // method here would re-create the precision the single grant exists
        // to refuse.
        return Some(Grant::Execute);
    }

    // --- edit: writes into a room, and thereby into a file -----------------
    let writes = matches!(method.as_str(), "PUT" | "POST" | "PATCH" | "DELETE");
    let editable = path.starts_with("/api/docs")
        || path.starts_with("/api/file")
        || path.starts_with("/api/scratchpad")
        || path.starts_with("/api/asset")
        || path.starts_with("/api/find/replace")
        || path.starts_with("/api/projects");
    if writes && editable {
        return Some(Grant::Edit);
    }
    // The live room. A websocket upgrade is a GET, and it is the write path
    // for every keystroke — treating it as a read because of its method would
    // hand `view` the ability to edit.
    if path.starts_with("/api/ws") {
        return Some(Grant::Edit);
    }

    // --- view: everything a reader sees ------------------------------------
    let reads = matches!(method.as_str(), "GET" | "HEAD");
    let readable = path.starts_with("/api/docs")
        || path.starts_with("/api/projects")
        || path.starts_with("/api/files")
        || path.starts_with("/api/file")
        || path.starts_with("/api/runs")
        || path.starts_with("/api/search")
        || path.starts_with("/api/find")
        || path.starts_with("/api/structure")
        || path.starts_with("/api/blame")
        || path.starts_with("/api/git")
        || path.starts_with("/api/sessions")
        || path.starts_with("/api/merged")
        || path.starts_with("/api/worktrees")
        || path == "/api/fleet"
        || path.starts_with("/api/executor")
        || path.starts_with("/api/health")
        || path.starts_with("/api/complete")
        || path.starts_with("/api/asset");
    if reads && readable {
        return Some(Grant::View);
    }

    None
}

/// Whether `machine` may make this request.
pub fn permitted(
    method: &str,
    path: &str,
    grants: &BTreeSet<Grant>,
    machine: &str,
) -> Result<Grant, Denial> {
    match required(method, path) {
        Some(grant) if grants.contains(&grant) => Ok(grant),
        Some(grant) => Err(Denial::needs(grant, method, path, machine)),
        None => Err(Denial::unknown(method, path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grants(list: &[Grant]) -> BTreeSet<Grant> {
        list.iter().copied().collect()
    }

    fn all() -> BTreeSet<Grant> {
        grants(&[Grant::View, Grant::Edit, Grant::Execute])
    }

    #[test]
    fn a_read_needs_view() {
        assert_eq!(
            permitted("GET", "/api/docs/abc", &grants(&[Grant::View]), "laptop"),
            Ok(Grant::View)
        );
        let denied = permitted("GET", "/api/docs/abc", &grants(&[]), "laptop").unwrap_err();
        assert_eq!(denied.needs, Some(Grant::View));
    }

    #[test]
    fn a_write_needs_edit_and_view_alone_will_not_do() {
        let denied =
            permitted("PUT", "/api/docs/abc", &grants(&[Grant::View]), "laptop").unwrap_err();
        assert_eq!(denied.needs, Some(Grant::Edit));
        assert!(
            denied.message.contains("hick fleet grant laptop edit"),
            "{denied:?}"
        );
    }

    #[test]
    fn the_live_room_needs_edit_even_though_it_is_a_get() {
        // A websocket upgrade is a GET and it is the write path for every
        // keystroke. Treating it as a read because of its method would hand
        // `view` the ability to edit.
        assert_eq!(
            permitted("GET", "/api/ws?doc=doc:abc", &grants(&[Grant::Edit]), "l"),
            Ok(Grant::Edit)
        );
        assert!(permitted("GET", "/api/ws?doc=doc:abc", &grants(&[Grant::View]), "l").is_err());
    }

    #[test]
    fn running_a_cell_needs_execute_and_not_merely_edit() {
        // A cell run is a POST to a document, so it would read as an ordinary
        // write if the execute routes were not matched first.
        for path in [
            "/api/docs/abc/run",
            "/api/docs/abc/check",
            "/api/docs/abc/agent",
            "/api/terminals",
            "/api/merged/write",
        ] {
            let denied =
                permitted("POST", path, &grants(&[Grant::View, Grant::Edit]), "box").unwrap_err();
            assert_eq!(denied.needs, Some(Grant::Execute), "{path}");
            assert_eq!(permitted("POST", path, &all(), "box"), Ok(Grant::Execute));
        }
    }

    #[test]
    fn listing_terminals_is_execute_too_because_the_family_is_the_boundary() {
        let denied =
            permitted("GET", "/api/terminals", &grants(&[Grant::View]), "box").unwrap_err();
        assert_eq!(denied.needs, Some(Grant::Execute));
    }

    #[test]
    fn a_query_string_cannot_change_the_verdict() {
        // A peer must not be able to talk its way past the table by appending
        // something to the path.
        let denied = permitted(
            "POST",
            "/api/docs/abc/run?pretend=read",
            &grants(&[Grant::View, Grant::Edit]),
            "box",
        )
        .unwrap_err();
        assert_eq!(denied.needs, Some(Grant::Execute));
    }

    #[test]
    fn an_unknown_route_is_refused_even_with_every_grant() {
        // Deny by default: allowing anything unmatched would make every route
        // added later reachable by every paired machine without anybody
        // deciding so.
        let denied = permitted("GET", "/api/something-new", &all(), "laptop").unwrap_err();
        assert_eq!(denied.needs, None);
        assert!(
            denied.message.contains("only routes it knows"),
            "{denied:?}"
        );
    }

    #[test]
    fn settings_are_not_reachable_over_the_channel_at_all() {
        // They hold provider keys and the continuity switch. The fleet channel
        // carries no key material by design rather than by rule, so there is
        // nothing here to enforce and nothing to get wrong.
        for method in ["GET", "PUT"] {
            let denied = permitted(method, "/api/settings/keys", &all(), "laptop").unwrap_err();
            assert_eq!(denied.needs, None, "{method}");
        }
        assert!(permitted("GET", "/api/settings/ui", &all(), "l").is_err());
    }

    #[test]
    fn a_peer_can_read_the_fleet_but_never_change_it() {
        // The escalation this must not have: a peer granting ITSELF `execute`
        // from inside the channel those grants exist to bound.
        assert_eq!(
            permitted("GET", "/api/fleet", &grants(&[Grant::View]), "laptop"),
            Ok(Grant::View)
        );
        for (method, path) in [
            ("POST", "/api/fleet/accept"),
            ("POST", "/api/fleet/grant"),
            ("DELETE", "/api/fleet/machine"),
            ("GET", "/api/fleet/invite"),
        ] {
            let denied = permitted(method, path, &all(), "laptop").unwrap_err();
            assert_eq!(denied.needs, None, "{method} {path}");
        }
    }

    #[test]
    fn a_denial_names_the_machine_and_the_command_that_fixes_it() {
        let denied = permitted(
            "POST",
            "/api/docs/x/run",
            &grants(&[Grant::View]),
            "agent-box",
        )
        .unwrap_err();
        assert!(denied.message.contains("agent-box"), "{denied:?}");
        assert!(
            denied
                .message
                .contains("hick fleet grant agent-box execute"),
            "{denied:?}"
        );
        // And says WHY it is off, rather than only that it is.
        assert!(
            denied.message.contains("my laptop was stolen"),
            "{denied:?}"
        );
    }
}
