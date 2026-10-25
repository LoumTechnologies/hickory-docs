<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-case/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `0d65dd7c5c6379a8d9945107e16351417e75ef3873c1e59fb30ffdbb14d48f63`.

<book:entry path="crates/hick-case/README.md">
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
</book:entry>

</book:doc>
