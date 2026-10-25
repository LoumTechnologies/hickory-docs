<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-condition/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `88eeb1abeec0948b6bfd3b41cd5d6893e107807b17f37a7b5291e16e21bf4843`.

<book:entry path="crates/hick-condition/README.md">
# hick-condition

Recursive-descent parser and evaluator for boolean conditions used in `when`
attributes and `<hick:when>` tags.

## Grammar

Supports `&&` (AND), `||` (OR), `!` (NOT), parentheses, `var=value`,
`var!=value`, and bare `var` (truthy if defined and non-empty).

## Key types

- `Condition` — the AST enum
- `VariableResolver` — trait for variable lookup, with a blanket impl for `HashMap<String, String>`
- `Condition::parse` — entry point for parsing a condition string
- `Condition::evaluate` — entry point for evaluating against a resolver

The `hick` crate delegates all conditional filtering to this crate.
</book:entry>

</book:doc>
