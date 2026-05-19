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
