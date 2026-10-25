<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# ACP Agents Work In The Agent Pane And Leave A Hickory Record

For: Authors using an agent to change documents while retaining review control.

Given an installed ACP v1 agent, when it is selected in the Agent pane, then
Hickory launches its executable directly in the workspace, negotiates its
capabilities, and connects a private loopback MCP server containing Hickory's
existing document tools. The downloaded desktop executable also supplies the
stdio proxy; a separate `hick` installation is not required.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/acp-agents-are-first-class.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
