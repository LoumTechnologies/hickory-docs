<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# New Project Offers The Scaffolder's Own Options, Not A List This Product Maintains

For: Authors editing executable documents and the files those documents produce.

Given a machine with a .NET SDK, when File → New Project is opened, then the
templates offered are exactly what `dotnet new list` reports on that machine,
and the fields shown for a chosen template are exactly the options `dotnet new
<template> --help` names — including templates from packages this product has
never heard of.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
