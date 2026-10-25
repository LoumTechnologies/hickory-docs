<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-secrets/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `1524fefb266f9bf7435e34394b981a2e5d5d57ac8f32b6033fc58d98065c4682`.

<book:entry path="crates/hick-secrets/README.md">
# hick-secrets

Pluggable secrets storage with a `SecretsProvider` async trait.

## Implementations

| Provider | Description |
|---|---|
| `AgeSecretsProvider` | Decrypts `<name>.age` files from a directory using an age X25519 identity key; runs on blocking thread pool |
| `EnvSecretsProvider` | Reads secrets from host environment variables |
| `SecretsProviderChain` | Tries multiple backends in order, falling through on `NotFound` |
| `SecretCache` | TTL-aware `DashMap` cache of `secrecy::SecretString` values that zeroize on drop |
| `InMemorySecretsProvider` | In-memory implementation for tests |

Secret values are always stored as `secrecy::SecretString` and zeroized when
dropped.
</book:entry>

</book:doc>
