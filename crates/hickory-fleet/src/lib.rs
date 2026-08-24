//! One engineer, many machines: identity, the mutual key list, and grants.
//!
//! See `docs/specs/freeform/one-engineer-many-machines.md`. One person has a
//! laptop, a desktop, a Windows box for the thing that only builds on Windows,
//! and increasingly a machine bought to run an agent on. This is the part that
//! must be right first: **nothing is reachable yet.**
//!
//! ## Keys, not accounts
//!
//! - **A machine is a keypair**, generated on first run, the private half
//!   never leaving the machine and never in the repository.
//! - **A fleet is a mutual list of public keys**, under the per-user state
//!   directory `hickory-workspace` already owns — deliberately not in the
//!   project, for the reason that module already gives: git would eventually
//!   commit it.
//! - **There is no account, no directory, and no server** to be signed in to.
//! - **Revocation is deleting a public key**, and it is complete, because no
//!   server holds a session you cannot reach.
//!
//! ## Enrolment is SSH's ceremony, without a rendezvous
//!
//! The design sketches a short one-time code (`7 QUAIL DRIFT 2`) that expires
//! in sixty seconds. A code that short cannot carry a public key, so it
//! assumes a rendezvous where the two machines find each other and exchange
//! keys under cover of the code — and the only rendezvous available to two
//! machines that cannot yet reach each other is one we would run.
//! `local-only.md` deletes that, and a relay is out of scope rather than
//! deferred.
//!
//! So the ceremony here is the same one without the rendezvous: an
//! **invitation** is self-contained — the machine's name and public key, with
//! a checksum — and the other machine accepts it. Mutual enrolment is both
//! machines doing both halves, which is exactly what `ssh-copy-id` in each
//! direction is. It needs no network at all, which is also why it works from a
//! café, over a phone screenshot, or read aloud.
//!
//! ## Grants are per machine, per verb
//!
//! `view` and `edit` are on; **`execute` is off by default** and granted per
//! peer. Three earlier grants (`run`, `terminal`, `agent`) collapsed into one
//! because they all mean "code runs on that machine as me", and three switches
//! implied a precision the security model does not have.
//!
//! Two things are deliberately **not** claimed. A peer with `edit` is not
//! harmless — it can write a document you later run, which is the same trust
//! as accepting a pull request. And a compromised machine with `execute` on an
//! unsandboxed peer is a full compromise of that peer: the grant model bounds
//! the blast radius, it does not defeat it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use base64::Engine as _;
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// What a peer is allowed to do here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grant {
    /// See documents, outputs, ribbons, transcripts, the running terminal's
    /// bytes. On by default.
    View,
    /// Write into the room, and thereby into the file. On by default — and
    /// not harmless: it can write a document you later run.
    Edit,
    /// Run a cell or the up-loop's pipeline, type into a shell, start or steer
    /// an agent turn. **Off by default**, because "my laptop was stolen"
    /// should not read as "every machine I own now executes whatever the thief
    /// types".
    Execute,
}

impl Grant {
    pub fn as_str(self) -> &'static str {
        match self {
            Grant::View => "view",
            Grant::Edit => "edit",
            Grant::Execute => "execute",
        }
    }

    /// What a newly paired machine gets.
    pub fn defaults() -> BTreeSet<Grant> {
        [Grant::View, Grant::Edit].into_iter().collect()
    }

    pub fn parse(value: &str) -> Result<Grant> {
        match value.trim().to_ascii_lowercase().as_str() {
            "view" => Ok(Grant::View),
            "edit" => Ok(Grant::Edit),
            "execute" => Ok(Grant::Execute),
            other => bail!(
                "unknown grant {other:?} — a grant is `view`, `edit` or `execute`.\n  \
                 `run`, `terminal` and `agent` were three separate grants once; they \
                 all mean \"code runs on that machine as me\", so they are one."
            ),
        }
    }
}

/// What kind of machine this is. A phone has no executor to grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Desktop,
    /// Reads and captures; never executes (`notes-ide.md`).
    Phone,
}

/// One machine in the fleet — a name, a public key, and what it may do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Machine {
    pub name: String,
    /// Base64 of the 32-byte ed25519 public key.
    pub public_key: String,
    #[serde(default)]
    pub kind: Kind,
    pub grants: BTreeSet<Grant>,
    /// When it was added, from the caller. There is no clock in here.
    #[serde(default)]
    pub added: String,
}

impl Machine {
    /// `SHA256:…`, the way ssh prints a key fingerprint — short enough to read
    /// aloud and compare, and the thing a person actually checks.
    pub fn fingerprint(&self) -> String {
        fingerprint_of(&self.public_key)
    }

    pub fn may(&self, grant: Grant) -> bool {
        self.grants.contains(&grant)
    }
}

/// The fingerprint of a base64 public key.
pub fn fingerprint_of(public_key: &str) -> String {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(public_key)
        .unwrap_or_default();
    let digest = Sha256::digest(&bytes);
    format!(
        "SHA256:{}",
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(digest)
    )
}

// ---------------------------------------------------------------------------
// This machine
// ---------------------------------------------------------------------------

/// This machine's own keypair.
///
/// The private half is written under the per-user state directory with owner-
/// only permissions on Unix. The design says "the platform keychain", and this
/// is not that: it is a file only this user can read. The difference matters
/// when the machine is compromised at the user level, and it is named here
/// rather than implied away.
pub struct Identity {
    pub name: String,
    signing: SigningKey,
    dir: PathBuf,
}

impl Identity {
    /// Load this machine's identity, generating one on first run.
    pub fn load_or_create(base: &Path, name: &str) -> Result<Self> {
        let dir = base.join("fleet");
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("could not create {}", dir.display()))?;
        let key_path = dir.join("machine.key");

        let signing = match std::fs::read_to_string(&key_path) {
            Ok(raw) => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(raw.trim())
                    .with_context(|| format!("{} is not readable as a key", key_path.display()))?;
                let bytes: [u8; 32] = bytes.as_slice().try_into().map_err(|_| {
                    anyhow::anyhow!(
                        "{} is not a 32-byte ed25519 private key. Delete it to \
                         generate a new identity — but every machine that paired \
                         with this one will have to pair again, because its key is \
                         how they know it.",
                        key_path.display()
                    )
                })?;
                SigningKey::from_bytes(&bytes)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let signing = SigningKey::generate(&mut rand_core::OsRng);
                let encoded = base64::engine::general_purpose::STANDARD.encode(signing.to_bytes());
                std::fs::write(&key_path, &encoded)
                    .with_context(|| format!("could not write {}", key_path.display()))?;
                restrict(&key_path)?;
                signing
            }
            Err(e) => {
                return Err(e).with_context(|| format!("reading {}", key_path.display()));
            }
        };

        Ok(Self {
            name: name.to_string(),
            signing,
            dir,
        })
    }

    /// The public half, base64.
    pub fn public_key(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.signing.verifying_key().to_bytes())
    }

    pub fn fingerprint(&self) -> String {
        fingerprint_of(&self.public_key())
    }

    /// The raw 32 bytes of the private half, for constructing the peer
    /// transport's endpoint identity.
    ///
    /// Exposing a private key is a smell, and the alternative is worse: the
    /// transport dials by public key, so its endpoint identity must be **this
    /// same key**. A second keypair would mean the machine has two
    /// identities — the one the fleet list holds and the one it actually
    /// connects with — and the list would then authenticate neither.
    ///
    /// It never leaves the process. Nothing serializes it, nothing logs it,
    /// and the only caller is the endpoint builder.
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing.to_bytes()
    }

    /// Prove possession of the private half over `challenge`.
    ///
    /// Unused by anything that talks over a network yet — nothing is reachable
    /// — but it is what makes the key an identity rather than a name, and it
    /// is here so the transport cannot be built without it.
    pub fn sign(&self, challenge: &[u8]) -> String {
        use ed25519_dalek::Signer as _;
        base64::engine::general_purpose::STANDARD.encode(self.signing.sign(challenge).to_bytes())
    }

    /// The invitation this machine hands to another.
    pub fn invitation(&self, kind: Kind) -> Invitation {
        Invitation {
            name: self.name.clone(),
            public_key: self.public_key(),
            kind,
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

/// Verify a signature made by `public_key` over `challenge`.
pub fn verify(public_key: &str, challenge: &[u8], signature: &str) -> bool {
    use ed25519_dalek::Verifier as _;
    let Ok(key_bytes) = base64::engine::general_purpose::STANDARD.decode(public_key) else {
        return false;
    };
    let Ok(key_bytes) = <[u8; 32]>::try_from(key_bytes.as_slice()) else {
        return false;
    };
    let Ok(key) = VerifyingKey::from_bytes(&key_bytes) else {
        return false;
    };
    let Ok(sig_bytes) = base64::engine::general_purpose::STANDARD.decode(signature) else {
        return false;
    };
    let Ok(sig_bytes) = <[u8; 64]>::try_from(sig_bytes.as_slice()) else {
        return false;
    };
    key.verify(challenge, &ed25519_dalek::Signature::from_bytes(&sig_bytes))
        .is_ok()
}

#[cfg(unix)]
fn restrict(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("could not restrict {}", path.display()))
}

#[cfg(not(unix))]
fn restrict(_path: &Path) -> Result<()> {
    // Windows inherits the per-user state directory's ACL, which is already
    // owner-only. Said here rather than left as an empty function nobody can
    // interpret.
    Ok(())
}

// ---------------------------------------------------------------------------
// Invitations
// ---------------------------------------------------------------------------

/// A self-contained invitation: everything the other machine needs, and
/// nothing it has to fetch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invitation {
    pub name: String,
    pub public_key: String,
    #[serde(default)]
    pub kind: Kind,
}

impl Invitation {
    /// The string a person copies, reads aloud, or screenshots.
    ///
    /// Prefixed so it is recognisable in a chat log, and checksummed so a
    /// truncated paste fails loudly instead of enrolling a key that is not the
    /// one the other machine holds.
    pub fn encode(&self) -> String {
        let body = serde_json::to_vec(self).expect("an invitation encodes");
        let sum = &Sha256::digest(&body)[..4];
        format!(
            "hick-fleet:{}:{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&body),
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sum),
        )
    }

    pub fn decode(text: &str) -> Result<Invitation> {
        let text = text.trim();
        let rest = text.strip_prefix("hick-fleet:").ok_or_else(|| {
            anyhow::anyhow!(
                "that is not a fleet invitation — one starts with `hick-fleet:`.\n  \
                 Next step: run `hick fleet invite` on the machine you are adding \
                 and paste the whole line it prints."
            )
        })?;
        let (body, sum) = rest
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("that invitation is truncated — it has no checksum"))?;
        let body = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(body)
            .context("that invitation is not readable — it was probably truncated")?;
        let sum = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(sum)
            .context("that invitation's checksum is not readable")?;
        if sum != Sha256::digest(&body)[..4] {
            bail!(
                "that invitation's checksum does not match, so it arrived \
                 damaged — a truncated paste is the usual cause.\n  \
                 Nothing was added. Copy the whole line, including the last \
                 characters after the final colon."
            );
        }
        let invitation: Invitation =
            serde_json::from_slice(&body).context("that invitation is not readable")?;
        if invitation.name.trim().is_empty() {
            bail!("that invitation names no machine");
        }
        // A key that is not a key is refused here rather than at first use,
        // where the failure would be a connection that will not open.
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&invitation.public_key)
            .context("that invitation's key is not readable")?;
        if bytes.len() != 32 {
            bail!("that invitation's key is not a 32-byte ed25519 public key");
        }
        Ok(invitation)
    }
}

// ---------------------------------------------------------------------------
// The fleet
// ---------------------------------------------------------------------------

/// The mutual list of public keys, under the per-user state directory.
pub struct Fleet {
    path: PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
struct FleetFile {
    #[serde(default)]
    machines: Vec<Machine>,
}

impl Fleet {
    pub fn at(dir: &Path) -> Self {
        Self {
            path: dir.join("fleet.json"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every machine, by name. An unreadable file is an empty fleet: nothing
    /// is reachable, which is the safe reading.
    pub fn machines(&self) -> Vec<Machine> {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|raw| serde_json::from_str::<FleetFile>(&raw).ok())
            .map(|f| f.machines)
            .unwrap_or_default()
    }

    fn save(&self, machines: Vec<Machine>) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("could not create {}", parent.display()))?;
        }
        let body =
            serde_json::to_string_pretty(&FleetFile { machines }).context("encoding the fleet")?;
        std::fs::write(&self.path, body)
            .with_context(|| format!("could not write {}", self.path.display()))?;
        restrict(&self.path)
    }

    /// Add a machine from an invitation.
    ///
    /// Re-adding the same key under the same name is idempotent and keeps the
    /// grants already set: pairing twice must not silently re-grant.
    pub fn add(&self, invitation: &Invitation, added: &str) -> Result<Machine> {
        let mut machines = self.machines();
        if let Some(existing) = machines.iter().find(|m| m.name == invitation.name) {
            if existing.public_key != invitation.public_key {
                bail!(
                    "a different machine is already called {:?} here \
                     ({}).\n  \
                     A name is how you tell your machines apart and a key is how \
                     they are identified; two keys under one name would make the \
                     first indistinguishable from an impostor. Nothing was \
                     changed.\n  \
                     Next steps: remove the old one with `hick fleet remove {}`, \
                     or invite this machine under a different name.",
                    existing.name,
                    existing.fingerprint(),
                    existing.name,
                );
            }
            return Ok(existing.clone());
        }
        let machine = Machine {
            name: invitation.name.clone(),
            public_key: invitation.public_key.clone(),
            kind: invitation.kind,
            grants: Grant::defaults(),
            added: added.to_string(),
        };
        machines.push(machine.clone());
        machines.sort_by(|a, b| a.name.cmp(&b.name));
        self.save(machines)?;
        Ok(machine)
    }

    /// Revocation is deleting a public key, and it is complete — no server
    /// holds a session you cannot reach.
    pub fn remove(&self, name: &str) -> Result<bool> {
        let mut machines = self.machines();
        let before = machines.len();
        machines.retain(|m| m.name != name);
        if machines.len() == before {
            return Ok(false);
        }
        self.save(machines)?;
        Ok(true)
    }

    /// Change one machine's grants.
    pub fn set_grant(&self, name: &str, grant: Grant, on: bool) -> Result<Machine> {
        let mut machines = self.machines();
        let machine = machines
            .iter_mut()
            .find(|m| m.name == name)
            .ok_or_else(|| anyhow::anyhow!("no machine called {name:?} is in this fleet"))?;
        if grant == Grant::Execute && on && machine.kind == Kind::Phone {
            bail!(
                "{name} is a phone, and a phone has no executor to grant. It \
                 reads and captures; it cannot spawn a subprocess, so there are \
                 no cells, no terminal, no LSP and no debugger on it.\n  \
                 Nothing was changed."
            );
        }
        if on {
            machine.grants.insert(grant);
        } else {
            machine.grants.remove(&grant);
        }
        let updated = machine.clone();
        self.save(machines)?;
        Ok(updated)
    }

    pub fn get(&self, name: &str) -> Option<Machine> {
        self.machines().into_iter().find(|m| m.name == name)
    }
}

/// A default machine name: the host's, which is what the person calls it.
pub fn default_machine_name() -> String {
    std::env::var("HICKORY_MACHINE_NAME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .or_else(|| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "this-machine".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(dir: &Path, name: &str) -> Identity {
        Identity::load_or_create(dir, name).unwrap()
    }

    #[test]
    fn a_machine_is_a_keypair_generated_once() {
        let dir = tempfile::tempdir().unwrap();
        let first = identity(dir.path(), "laptop");
        let key = first.public_key();
        // Loading again is the SAME machine: an identity that changed on
        // restart would make every peer treat it as an impostor.
        let second = identity(dir.path(), "laptop");
        assert_eq!(second.public_key(), key);
        assert!(first.fingerprint().starts_with("SHA256:"));
    }

    #[test]
    fn the_private_half_never_leaves_the_machine() {
        let dir = tempfile::tempdir().unwrap();
        let id = identity(dir.path(), "laptop");
        // What travels is the invitation, and it carries the PUBLIC half only.
        let text = id.invitation(Kind::Desktop).encode();
        let private = std::fs::read_to_string(dir.path().join("fleet/machine.key")).unwrap();
        assert!(!text.contains(private.trim()));
        let back = Invitation::decode(&text).unwrap();
        assert_eq!(back.public_key, id.public_key());
    }

    #[test]
    fn a_key_proves_possession_and_a_wrong_key_does_not() {
        let dir = tempfile::tempdir().unwrap();
        let id = identity(dir.path(), "laptop");
        let sig = id.sign(b"challenge");
        assert!(verify(&id.public_key(), b"challenge", &sig));
        assert!(!verify(&id.public_key(), b"different", &sig));

        let other_dir = tempfile::tempdir().unwrap();
        let other = identity(other_dir.path(), "desktop");
        assert!(!verify(&other.public_key(), b"challenge", &sig));
    }

    #[test]
    fn a_truncated_invitation_fails_loudly_rather_than_enrolling_the_wrong_key() {
        let dir = tempfile::tempdir().unwrap();
        let id = identity(dir.path(), "laptop");
        let text = id.invitation(Kind::Desktop).encode();
        let cut = &text[..text.len() - 3];
        let err = Invitation::decode(cut).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("truncated") || msg.contains("checksum"),
            "{msg}"
        );
    }

    #[test]
    fn something_that_is_not_an_invitation_says_what_one_looks_like() {
        let err = Invitation::decode("hello").unwrap_err();
        assert!(format!("{err:#}").contains("hick-fleet:"));
        assert!(format!("{err:#}").contains("hick fleet invite"));
    }

    #[test]
    fn a_paired_machine_gets_view_and_edit_but_never_execute() {
        // "My laptop was stolen" must not read as "every machine I own now
        // executes whatever the thief types".
        let dir = tempfile::tempdir().unwrap();
        let id = identity(dir.path(), "desktop");
        let fleet = Fleet::at(dir.path());
        let added = fleet
            .add(&id.invitation(Kind::Desktop), "2026-08-23")
            .unwrap();
        assert!(added.may(Grant::View));
        assert!(added.may(Grant::Edit));
        assert!(!added.may(Grant::Execute));
    }

    #[test]
    fn execute_is_granted_deliberately_and_can_be_taken_back() {
        let dir = tempfile::tempdir().unwrap();
        let id = identity(dir.path(), "agent-box");
        let fleet = Fleet::at(dir.path());
        fleet
            .add(&id.invitation(Kind::Desktop), "2026-08-23")
            .unwrap();
        let granted = fleet.set_grant("agent-box", Grant::Execute, true).unwrap();
        assert!(granted.may(Grant::Execute));
        let revoked = fleet.set_grant("agent-box", Grant::Execute, false).unwrap();
        assert!(!revoked.may(Grant::Execute));
    }

    #[test]
    fn a_phone_cannot_be_given_execute_because_it_has_no_executor() {
        let dir = tempfile::tempdir().unwrap();
        let id = identity(dir.path(), "phone");
        let fleet = Fleet::at(dir.path());
        fleet
            .add(&id.invitation(Kind::Phone), "2026-08-23")
            .unwrap();
        let err = fleet.set_grant("phone", Grant::Execute, true).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("no executor to grant"), "{msg}");
        assert!(!fleet.get("phone").unwrap().may(Grant::Execute));
    }

    #[test]
    fn pairing_twice_is_idempotent_and_does_not_re_grant() {
        let dir = tempfile::tempdir().unwrap();
        let id = identity(dir.path(), "desktop");
        let fleet = Fleet::at(dir.path());
        fleet
            .add(&id.invitation(Kind::Desktop), "2026-08-23")
            .unwrap();
        fleet.set_grant("desktop", Grant::Edit, false).unwrap();
        // Re-pairing must not silently restore a grant somebody took away.
        fleet
            .add(&id.invitation(Kind::Desktop), "2026-08-24")
            .unwrap();
        assert!(!fleet.get("desktop").unwrap().may(Grant::Edit));
        assert_eq!(fleet.machines().len(), 1);
    }

    #[test]
    fn two_keys_under_one_name_is_refused_rather_than_silently_replaced() {
        // A name is how you tell your machines apart; a key is how they are
        // identified. Two keys under one name makes the first
        // indistinguishable from an impostor.
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let first = identity(a.path(), "desktop");
        let second = identity(b.path(), "desktop");
        let fleet = Fleet::at(a.path());
        fleet
            .add(&first.invitation(Kind::Desktop), "2026-08-23")
            .unwrap();
        let err = fleet
            .add(&second.invitation(Kind::Desktop), "2026-08-23")
            .unwrap_err();
        assert!(format!("{err:#}").contains("already called"));
        assert_eq!(fleet.get("desktop").unwrap().public_key, first.public_key());
    }

    #[test]
    fn revocation_is_deleting_a_key_and_it_is_complete() {
        let dir = tempfile::tempdir().unwrap();
        let id = identity(dir.path(), "stolen-laptop");
        let fleet = Fleet::at(dir.path());
        fleet
            .add(&id.invitation(Kind::Desktop), "2026-08-23")
            .unwrap();
        assert!(fleet.remove("stolen-laptop").unwrap());
        assert!(fleet.get("stolen-laptop").is_none());
        // Removing something that is not there is not an error, it is a no.
        assert!(!fleet.remove("stolen-laptop").unwrap());
    }

    #[test]
    fn an_unreadable_fleet_is_an_empty_one() {
        // Nothing is reachable, which is the safe reading.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("fleet.json"), "{not json").unwrap();
        assert!(Fleet::at(dir.path()).machines().is_empty());
    }
}
