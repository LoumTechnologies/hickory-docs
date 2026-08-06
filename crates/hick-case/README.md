# hick-case

String transformation utilities for identifier case conversions.

`detect_words` splits an identifier into words regardless of convention
(PascalCase, camelCase, snake_case, kebab-case, SCREAMING_SNAKE, acronyms).
Individual converters (`to_pascal_case`, `to_snake_case`, `to_kebab_case`, etc.)
rebuild the identifier in the target style.

The key function is `generate_case_variants(pattern, value)`, which produces all
case-transformed `(pattern, replacement)` pairs sorted by length. This powers
the `<hick:substitute variants="true">` feature, ensuring that when an
identifier is renamed in a generated file, every casing convention used
throughout the output is updated consistently.
