# hick-api

Gateway HTTP API served inside each WASM container sandbox.

Listens at `10.0.1.1:8080` and implements `HttpFilter` via `GatewayFilter`.
Intercepts three endpoints:

| Endpoint | Purpose |
|---|---|
| `POST /api/token/attenuate` | Narrow a capability token (strictly additive restrictions) |
| `POST /api/token/verify` | Verify a capability token is valid and unexpired |
| `POST /api/confirm` | Request human-in-the-loop approval for an action |

Uses macaroon-based `TokenAuthority` and `CapabilityToken` types from
`hick-token`. Human confirmation is delegated to the `ConfirmHandler` trait;
the default implementation is `AutoConfirm` for unattended pipelines.

This is the security boundary through which sandboxed containers request
narrowed capabilities or human approval before taking external actions.
