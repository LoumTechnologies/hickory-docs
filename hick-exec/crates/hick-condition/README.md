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
