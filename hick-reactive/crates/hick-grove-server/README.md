# hick-grove-server

Axum HTTP/WebSocket server that exposes a `GroveEngine` instance over a REST +
WebSocket API.

## REST routes

| Route | Purpose |
|---|---|
| `POST /documents` | Create a new document |
| `POST /documents/{id}/xml` | Apply a structured XML message |
| `POST /documents/{id}/update` | Apply a raw Yrs binary update |
| `GET /documents/{id}/state-vector` | Retrieve the Yrs state vector |
| `GET /documents/{id}/diff` | Encode a diff from a given state vector |

## WebSocket

`/documents/{id}/ws` — real-time collaborative sync endpoint for Yrs clients.

At startup, wires a `TasksHandler` (backed by an in-memory SQLite store) into
the `NamespaceRegistry`, demonstrating how grove plugins are composed into a
running service.
