<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Every Desktop Platform Can Confine A Cell

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given `HICKORY_EXECUTOR=sandbox` on Linux, macOS **or Windows**, when a
document's cell runs, then it may write only its own workdir, reaches the
network only if the document declared it, and cannot outlive the run that
started it. Where nothing can enforce that, the executor **refuses to run**
rather than running the cell unconfined.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/every-desktop-platform-can-confine-a-cell.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-bfbb07ecda29"></a>
## `crates/hickory-executor-sandbox/src/appcontainer.rs`

Related contracts: [A Sandboxed Cell Cannot Reach Past Its Workdir](../a-sandboxed-cell-cannot-reach-past-its-workdir/contract.md), [Every Desktop Platform Can Confine A Cell](contract.md).

<a id="f-bfbb07ecda29-Confinement"></a>
### `Confinement`

Windows confinement: AppContainer, a restricted-by-default token, and a
job object.
## Why this exists rather than a `bwrap`-shaped wrapper
On Linux and macOS the sandbox is a program you exec: `bwrap …` or
`sandbox-exec …` takes the command as arguments and confines what it
spawns. Windows has no such program. AppContainer is applied by the
*parent* at process-creation time — a process cannot put itself into one —
so something has to call `CreateProcessW` with the right attributes.
That something is `hick` itself, re-invoked as the hidden
`__sandbox-run` subcommand (see `hickory-cli`). It keeps the shape the
other platforms have — [`crate::policy::wrap`] still returns a program and
arguments — while putting the Win32 work somewhere first-party rather than
asking the user to install a launcher.
## The policy, and how each piece enforces it
* **Writes.** An AppContainer runs at Low integrity with its own package
SID, and by default that SID has access to nothing. The workdir is made
writable by granting *that SID specifically* an ACE on the directory.
Nothing else on the disk carries an ACE for it, so nothing else is
writable — this is the same guarantee bubblewrap gets from a read-only
bind, arrived at from the opposite direction.
* **Reads.** Windows grants `ALL APPLICATION PACKAGES` read access to the
system directories, so interpreters installed machine-wide work. One
installed per-user under `%LOCALAPPDATA%` may NOT — see the caveats.
* **Network.** An AppContainer with no capability SIDs has no network at
all. `internetClient` is added only when the document declared it.
* **Processes.** A job object with `KILL_ON_JOB_CLOSE` means a cell cannot
outlive the run that started it, which is what `--die-with-parent` does
on Linux.
## What this cannot do
It cannot hide the user's files from *reading* the way an empty `$HOME`
tmpfs does on Linux. AppContainer denies by default, so a profile with no
grants cannot read `%USERPROFILE%` either — but where Linux hides SSH keys
behind a mount, here they are merely unreadable, and a future grant added
for some other reason could expose them. The difference is written down
rather than smoothed over.

<storycode:copy id="f-bfbb07ecda29-Confinement">
//! Windows confinement: AppContainer, a restricted-by-default token, and a
//! job object.
//!
//! ## Why this exists rather than a `bwrap`-shaped wrapper
//!
//! On Linux and macOS the sandbox is a program you exec: `bwrap …` or
//! `sandbox-exec …` takes the command as arguments and confines what it
//! spawns. Windows has no such program. AppContainer is applied by the
//! *parent* at process-creation time — a process cannot put itself into one —
//! so something has to call `CreateProcessW` with the right attributes.
//!
//! That something is `hick` itself, re-invoked as the hidden
//! `__sandbox-run` subcommand (see `hickory-cli`). It keeps the shape the
//! other platforms have — [`crate::policy::wrap`] still returns a program and
//! arguments — while putting the Win32 work somewhere first-party rather than
//! asking the user to install a launcher.
//!
//! ## The policy, and how each piece enforces it
//!
//! * **Writes.** An AppContainer runs at Low integrity with its own package
//!   SID, and by default that SID has access to nothing. The workdir is made
//!   writable by granting *that SID specifically* an ACE on the directory.
//!   Nothing else on the disk carries an ACE for it, so nothing else is
//!   writable — this is the same guarantee bubblewrap gets from a read-only
//!   bind, arrived at from the opposite direction.
//! * **Reads.** Windows grants `ALL APPLICATION PACKAGES` read access to the
//!   system directories, so interpreters installed machine-wide work. One
//!   installed per-user under `%LOCALAPPDATA%` may NOT — see the caveats.
//! * **Network.** An AppContainer with no capability SIDs has no network at
//!   all. `internetClient` is added only when the document declared it.
//! * **Processes.** A job object with `KILL_ON_JOB_CLOSE` means a cell cannot
//!   outlive the run that started it, which is what `--die-with-parent` does
//!   on Linux.
//!
//! ## What this cannot do
//!
//! It cannot hide the user's files from *reading* the way an empty `$HOME`
//! tmpfs does on Linux. AppContainer denies by default, so a profile with no
//! grants cannot read `%USERPROFILE%` either — but where Linux hides SSH keys
//! behind a mount, here they are merely unreadable, and a future grant added
//! for some other reason could expose them. The difference is written down
//! rather than smoothed over.

#![cfg(windows)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use anyhow::{Context, Result, bail};
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Security::Authorization::{
    EXPLICIT_ACCESS_W, GRANT_ACCESS, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, SET_ACCESS,
    SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_GROUP, TRUSTEE_IS_SID, TRUSTEE_W,
};
use windows::Win32::Security::Isolation::{
    CreateAppContainerProfile, DeriveAppContainerSidFromAppContainerName,
};
use windows::Win32::Security::{
    ACL, DACL_SECURITY_INFORMATION, DeriveCapabilitySidsFromName, PSID, SECURITY_CAPABILITIES,
};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_BASIC_LIMIT_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows::Win32::System::Threading::{
    CREATE_NEW_PROCESS_GROUP, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
    GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, PROCESS_INFORMATION, STARTUPINFOEXW,
    UpdateProcThreadAttribute, WaitForSingleObject,
};
use windows::core::{PCWSTR, PWSTR};

/// Everything the confined command needs, from the caller's point of view.
pub struct Confinement<'a> {
    /// The one directory the command may write. Also its working directory.
    pub workdir: &'a Path,
    /// The command line, run through `cmd.exe /C` so a cell's shell syntax
    /// behaves as it does elsewhere.
    pub command: &'a str,
    /// Whether the document granted network access.
    pub allow_network: bool,
}

</storycode:copy>
<a id="f-bfbb07ecda29-run"></a>
### `run`

Run `command` inside an AppContainer and return its exit code.
Blocking: the caller is `hick __sandbox-run`, a process that exists only
to be this launcher, and whose own exit code is the command's.

<storycode:copy id="f-bfbb07ecda29-run">
/// Run `command` inside an AppContainer and return its exit code.
///
/// Blocking: the caller is `hick __sandbox-run`, a process that exists only
/// to be this launcher, and whose own exit code is the command's.
pub fn run(confinement: Confinement<'_>) -> Result<u32> {
    let name = container_name(confinement.workdir);
    let sid = ensure_profile(&name)?;
    grant_workdir(confinement.workdir, sid).with_context(|| {
        format!(
            "granting the sandbox write access to {:?}",
            confinement.workdir
        )
    })?;

    let mut capability_sids = Vec::new();
    if confinement.allow_network {
        // The single capability that means "may make outbound connections".
        // Nothing else is ever added: a sandbox whose grants grow silently is
        // not a sandbox.
        capability_sids.push(capability_sid("internetClient")?);
    }
    spawn_confined(&name, sid, &capability_sids, &confinement)
}

</storycode:copy>
<a id="f-bfbb07ecda29-container_name"></a>
### `container_name`

A per-workdir container name, stable for the same directory.
Stable so repeated runs of one container reuse one profile rather than
littering the registry; per-workdir so two containers cannot see each
other's files, which is the cross-container isolation the Linux path gets
from binding only one directory.

<storycode:copy id="f-bfbb07ecda29-container_name">
/// A per-workdir container name, stable for the same directory.
///
/// Stable so repeated runs of one container reuse one profile rather than
/// littering the registry; per-workdir so two containers cannot see each
/// other's files, which is the cross-container isolation the Linux path gets
/// from binding only one directory.
fn container_name(workdir: &Path) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in workdir.to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    // The name is limited to 64 characters and must be a valid package
    // family name; hex of a hash is both.
    format!("hickory.cell.{hash:016x}")
}

</storycode:copy>
<a id="f-bfbb07ecda29-wide"></a>
### `wide`

<storycode:copy id="f-bfbb07ecda29-wide">
fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

</storycode:copy>
<a id="f-bfbb07ecda29-ensure_profile"></a>
### `ensure_profile`

Create the profile, or derive the SID of the one already there.

<storycode:copy id="f-bfbb07ecda29-ensure_profile">
/// Create the profile, or derive the SID of the one already there.
fn ensure_profile(name: &str) -> Result<PSID> {
    let wide_name = wide(name);
    let display = wide("Hickory Docs cell");
    let description = wide("Confined execution of a hick document's cell");

    // SAFETY: all four pointers are to buffers that outlive the call, and the
    // capability list is explicitly empty (null, 0), which is what gives the
    // container no privileges at all.
    let created = unsafe {
        CreateAppContainerProfile(
            PCWSTR(wide_name.as_ptr()),
            PCWSTR(display.as_ptr()),
            PCWSTR(description.as_ptr()),
            // No capabilities at all: this is what leaves the container able
            // to reach nothing until an ACE or a capability SID says so.
            None,
        )
    };
    match created {
        Ok(sid) => Ok(sid),
        Err(error) if error.code() == ERROR_ALREADY_EXISTS.to_hresult() => {
            // Already created by an earlier run of the same container. The
            // profile is reusable; only the SID is needed.
            // SAFETY: `wide_name` outlives the call.
            let sid =
                unsafe { DeriveAppContainerSidFromAppContainerName(PCWSTR(wide_name.as_ptr())) }
                    .context("deriving the SID of an existing AppContainer profile")?;
            Ok(sid)
        }
        Err(error) => Err(error).context(
            "creating an AppContainer profile. Windows 8 or later is required; on a machine \
             where AppContainer is unavailable, run cells under WSL2 or use \
             HICKORY_EXECUTOR=docker",
        ),
    }
}

</storycode:copy>
<a id="f-bfbb07ecda29-capability_sid"></a>
### `capability_sid`

The SID for a named capability, e.g. `internetClient`.

<storycode:copy id="f-bfbb07ecda29-capability_sid">
/// The SID for a named capability, e.g. `internetClient`.
fn capability_sid(name: &str) -> Result<PSID> {
    let wide_name = wide(name);
    let mut group_sids: *mut PSID = std::ptr::null_mut();
    let mut group_count = 0u32;
    let mut sids: *mut PSID = std::ptr::null_mut();
    let mut count = 0u32;
    // SAFETY: out parameters are all valid pointers; the returned arrays are
    // owned by the OS and intentionally not freed — this process is about to
    // exit, and freeing them would invalidate the SID handed to CreateProcess.
    unsafe {
        DeriveCapabilitySidsFromName(
            PCWSTR(wide_name.as_ptr()),
            &mut group_sids,
            &mut group_count,
            &mut sids,
            &mut count,
        )
    }
    .with_context(|| format!("deriving the capability SID for {name}"))?;
    if count == 0 || sids.is_null() {
        bail!("Windows returned no capability SID for '{name}'");
    }
    // SAFETY: `count > 0` was just checked, so the first element exists.
    Ok(unsafe { *sids })
}

</storycode:copy>
<a id="f-bfbb07ecda29-grant_workdir"></a>
### `grant_workdir`

Give the container's SID write access to exactly one directory.
This is the whole write policy: the container starts able to write
nothing, and this adds the single exception. It ADDS to the existing DACL
rather than replacing it, so the user's own access to their directory is
untouched.

<storycode:copy id="f-bfbb07ecda29-grant_workdir">
/// Give the container's SID write access to exactly one directory.
///
/// This is the whole write policy: the container starts able to write
/// nothing, and this adds the single exception. It ADDS to the existing DACL
/// rather than replacing it, so the user's own access to their directory is
/// untouched.
fn grant_workdir(workdir: &Path, sid: PSID) -> Result<()> {
    let mut access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: 0x1F01FF, // FILE_ALL_ACCESS
        grfAccessMode: SET_ACCESS,
        // Inherited by files and subdirectories created later: a cell that
        // makes a directory must be able to write inside it.
        grfInheritance: windows::Win32::Security::ACE_FLAGS(0x3), // OBJECT|CONTAINER_INHERIT
        Trustee: TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_GROUP,
            ptstrName: PWSTR(sid.0 as *mut u16),
        },
    };
    access.grfAccessMode = GRANT_ACCESS;

    let mut new_acl: *mut ACL = std::ptr::null_mut();
    // SAFETY: one entry, a valid out pointer, and no existing ACL to merge
    // into (None) — the ACL produced is the exception list, applied below
    // with SetNamedSecurityInfoW, which merges it into the object's DACL.
    unsafe { SetEntriesInAclW(Some(&[access]), None, &mut new_acl) }
        .ok()
        .context("building the access-control entry for the sandbox")?;

    let mut path = wide(&workdir.to_string_lossy());
    // SAFETY: `path` and `new_acl` both outlive the call.
    let result = unsafe {
        SetNamedSecurityInfoW(
            PWSTR(path.as_mut_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(new_acl),
            None,
        )
    };
    result
        .ok()
        .context("applying the sandbox's access to the workdir")?;
    Ok(())
}

</storycode:copy>
<a id="f-bfbb07ecda29-spawn_confined"></a>
### `spawn_confined`

`CreateProcessW` with the security capabilities attached, inside a job.

<storycode:copy id="f-bfbb07ecda29-spawn_confined">
/// `CreateProcessW` with the security capabilities attached, inside a job.
fn spawn_confined(
    name: &str,
    sid: PSID,
    capabilities: &[PSID],
    confinement: &Confinement<'_>,
) -> Result<u32> {
    let _ = name;
    let mut security = SECURITY_CAPABILITIES {
        AppContainerSid: sid,
        Capabilities: if capabilities.is_empty() {
            std::ptr::null_mut()
        } else {
            // The array is borrowed for the duration of CreateProcessW only.
            capabilities.as_ptr() as *mut _
        },
        CapabilityCount: capabilities.len() as u32,
        Reserved: 0,
    };

    // Size the attribute list, then fill it. The first call is expected to
    // fail with ERROR_INSUFFICIENT_BUFFER; only the size it writes is used.
    let mut size = 0usize;
    // SAFETY: the documented two-call sizing protocol.
    unsafe {
        let _ = InitializeProcThreadAttributeList(
            LPPROC_THREAD_ATTRIBUTE_LIST(std::ptr::null_mut()),
            1,
            0,
            &mut size,
        );
    }
    let mut buffer = vec![0u8; size];
    let attributes = LPPROC_THREAD_ATTRIBUTE_LIST(buffer.as_mut_ptr() as *mut _);
    // SAFETY: `buffer` is exactly the size the sizing call asked for.
    unsafe { InitializeProcThreadAttributeList(attributes, 1, 0, &mut size) }
        .context("initialising the process attribute list")?;

    // SAFETY: `security` outlives the CreateProcessW call below, which is the
    // lifetime the attribute list requires of it.
    unsafe {
        UpdateProcThreadAttribute(
            attributes,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            Some(&mut security as *mut _ as *mut _),
            std::mem::size_of::<SECURITY_CAPABILITIES>(),
            None,
            None,
        )
    }
    .context("attaching the AppContainer to the process attributes")?;

    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.lpAttributeList = attributes;

    // `cmd.exe /C` for the same reason `sh -c` is used elsewhere: a cell's
    // own shell syntax must behave exactly as it does unsandboxed.
    let mut command_line = wide(&format!("cmd.exe /C {}", confinement.command));
    let workdir = wide(&confinement.workdir.to_string_lossy());
    let mut process = PROCESS_INFORMATION::default();
    let overrides = crate::policy::runtime_environment(confinement.workdir);
    let mut environment: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os()
        .filter(|(name, _)| {
            !overrides
                .iter()
                .any(|(key, _)| name.to_string_lossy().eq_ignore_ascii_case(key))
        })
        .collect();
    environment.extend(
        overrides
            .into_iter()
            .map(|(name, value)| (name.into(), value.into())),
    );
    environment.sort_by_key(|(name, _)| name.to_string_lossy().to_uppercase());
    let mut environment: Vec<u16> = environment
        .into_iter()
        .flat_map(|(name, value)| {
            let mut entry = name.encode_wide().collect::<Vec<_>>();
            entry.push(b'=' as u16);
            entry.extend(value.encode_wide());
            entry.push(0);
            entry
        })
        .collect();
    environment.push(0);

    // SAFETY: every pointer outlives the call; the command line is mutable as
    // CreateProcessW requires.
    let created = unsafe {
        windows::Win32::System::Threading::CreateProcessW(
            None,
            PWSTR(command_line.as_mut_ptr()),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT
                | CREATE_NEW_PROCESS_GROUP
                | windows::Win32::System::Threading::CREATE_UNICODE_ENVIRONMENT,
            Some(environment.as_ptr().cast()),
            PCWSTR(workdir.as_ptr()),
            &startup.StartupInfo,
            &mut process,
        )
    };
    // SAFETY: the list was initialised above and is not used after this.
    unsafe { DeleteProcThreadAttributeList(attributes) };
    created.context(
        "starting the confined process. A cell that cannot start here usually means the \
         interpreter is installed per-user rather than machine-wide, where an AppContainer \
         cannot read it",
    )?;

    let job = assign_job(process.hProcess)?;

    // SAFETY: a valid process handle from CreateProcessW.
    let waited = unsafe { WaitForSingleObject(process.hProcess, INFINITE) };
    if waited != WAIT_OBJECT_0 {
        bail!("waiting for the confined process failed");
    }
    let mut code = 0u32;
    // SAFETY: same handle, valid out pointer.
    unsafe { GetExitCodeProcess(process.hProcess, &mut code) }
        .context("reading the confined process's exit code")?;

    // SAFETY: handles created above, each closed exactly once.
    unsafe {
        let _ = CloseHandle(process.hThread);
        let _ = CloseHandle(process.hProcess);
        let _ = CloseHandle(job);
    }
    Ok(code)
}

</storycode:copy>
<a id="f-bfbb07ecda29-assign_job"></a>
### `assign_job`

Put the process in a job that dies with us.
Without this a cell that spawns a background process leaves it running
after the run finishes, holding files open in a workdir we are about to
delete. `--die-with-parent` is the Linux spelling of the same rule.

<storycode:copy id="f-bfbb07ecda29-assign_job">
/// Put the process in a job that dies with us.
///
/// Without this a cell that spawns a background process leaves it running
/// after the run finishes, holding files open in a workdir we are about to
/// delete. `--die-with-parent` is the Linux spelling of the same rule.
fn assign_job(process: HANDLE) -> Result<HANDLE> {
    // SAFETY: an unnamed job object; the handle is closed by the caller.
    let job =
        unsafe { CreateJobObjectW(None, PCWSTR::null()) }.context("creating the job object")?;
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
            LimitFlags: JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            ..Default::default()
        },
        ..Default::default()
    };
    // SAFETY: the struct matches the information class being set.
    unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &mut limits as *mut _ as *mut _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    }
    .context("setting the job object's limits")?;
    // SAFETY: both handles are valid.
    unsafe { AssignProcessToJobObject(job, process) }
        .context("assigning the confined process to its job")?;
    Ok(job)
}

</storycode:copy>
<a id="f-bfbb07ecda29-tests"></a>
### `tests`

<storycode:copy id="f-bfbb07ecda29-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_container_name_is_stable_and_per_directory() {
        // Stable, so repeated runs reuse one profile instead of littering the
        // registry; per-directory, so two containers cannot reach each
        // other's files.
        let a = container_name(Path::new(r"C:\work\a"));
        assert_eq!(a, container_name(Path::new(r"C:\work\a")));
        assert_ne!(a, container_name(Path::new(r"C:\work\b")));
        assert!(a.starts_with("hickory.cell."));
        assert!(a.len() <= 64, "package family names are capped at 64: {a}");
    }
}
</storycode:copy>

### Assemble `crates/hickory-executor-sandbox/src/appcontainer.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-sandbox/src/appcontainer.rs"><storycode:paste select="#f-bfbb07ecda29-Confinement"/><storycode:paste select="#f-bfbb07ecda29-run"/><storycode:paste select="#f-bfbb07ecda29-container_name"/><storycode:paste select="#f-bfbb07ecda29-wide"/><storycode:paste select="#f-bfbb07ecda29-ensure_profile"/><storycode:paste select="#f-bfbb07ecda29-capability_sid"/><storycode:paste select="#f-bfbb07ecda29-grant_workdir"/><storycode:paste select="#f-bfbb07ecda29-spawn_confined"/><storycode:paste select="#f-bfbb07ecda29-assign_job"/><storycode:paste select="#f-bfbb07ecda29-tests"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-executor-sandbox/src/appcontainer.rs](contract.md#output-bfbb07ecda29)
- [crates/hickory-executor-sandbox/src/lib.rs](../../../mechanisms/crates/hickory-executor-sandbox/src/lib.rs.md#output-3ed8f409da24)
- [crates/hickory-executor-sandbox/src/policy.rs](../../../mechanisms/crates/hickory-executor-sandbox/src/policy.rs.md#output-875ceba98dc8)
- [crates/hickory-executor/src/lib.rs](../../../mechanisms/crates/hickory-executor/src/lib.rs.md#output-87ce480a6eab)

### Tests and fixtures

- [crates/hickory-executor-sandbox/tests/confinement.rs](../../../mechanisms/crates/hickory-executor-sandbox/tests/confinement.rs.md#output-c8a04313c63c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
