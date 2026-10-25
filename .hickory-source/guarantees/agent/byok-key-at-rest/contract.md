<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Historical hosted-account key encryption

For: Authors using an agent to change documents while retaining review control.

This contract requires apps/server/src/keyvault.rs, account rows, Postgres and a deployment encryption key. Those implementations are absent from the captured source. The local key-store contract is agent/keys-are-stored-locally-and-never-leave.md; it explicitly describes plaintext protected by local file permissions. Do not advertise hosted encryption for that store.

This is historical product intent. Preserved source files below remain part of byte-exact construction and may serve other current behaviors.

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/byok-key-at-rest.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: This contract requires apps/server/src/keyvault.rs, account rows, Postgres and a deployment encryption key. Those implementations are absent from the captured source. The local key-store contract is agent/keys-are-stored-locally-and-never-leave.md; it explicitly describes plaintext protected by local file permissions. Do not advertise hosted encryption for that store.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
