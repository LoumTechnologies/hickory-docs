<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Bind the document language before quoting code

For: Authors embedding source code or Hickory examples in a literate document.

A chapter using the book prefix can quote hick-prefixed examples without treating those examples as construction acts. Its explicit root selects the prefix before any quoted namespace declaration.

## What must happen

Given an explicit root binding a prefix to the Hickory namespace, tags in that prefix are parsed as Hickory structure. Other tag prefixes are ordinary text. No XML entity-escaping mechanism is required for ordinary raw text. Raw payload elements still terminate at their own literal closing delimiter, and Markdown fences and comments follow the parser's explicit rules.


## Limits of the evidence

Namespace discovery scans text before normal child parsing. Bare documents quoting another namespace can select an unintended prefix; the rewrite avoids this by always binding its prefix explicitly. Parser tests do not establish all editor rendering behavior.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/prefix-only-parsing-no-escaping.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the no-escaping model while naming the actual delimiter, comment and fence boundaries instead of promising every byte is raw everywhere.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
