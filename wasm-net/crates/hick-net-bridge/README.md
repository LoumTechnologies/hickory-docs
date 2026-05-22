# bridge

Minimal in-process Ethernet frame router for inter-container networking.

`NetworkBridge` maps VM IP addresses to Tokio `mpsc::Sender<Vec<u8>>` channels,
enabling the network loop of one container to deliver Ethernet frames directly
to another without leaving the process.

Also provides internal DNS resolution by container name, so containers can
address each other by name rather than IP.

No heavy dependencies beyond Tokio's `sync` feature.
