---
id: m-0
title: "Agent Architecture Bifurcation"
---

## Description

Split the monolithic hick-agent into a generic agent SDK (zero hick knowledge) and extension/binary crates that layer hick-specific behavior on top. This enables building and evaluating many different agent behaviors by mixing and matching extensions.
