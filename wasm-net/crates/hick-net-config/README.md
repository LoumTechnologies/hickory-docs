# net-config

Shared network constants and per-container address configuration for the virtual
network.

Defines the default subnet (192.168.127.0/24, gateway .1, VM .3) and MAC
addresses matching the upstream `c2w-net` bridge.

`NetworkConfig::for_container(index)` generates a unique `10.0.{index+1}.0/24`
subnet per container for multi-container setups.

Used by both `network` and `tcp-nat` to stay in sync on addressing without
duplicating constants.
