# hickory-xml

Parser-agnostic XML DOM abstraction layer backed by `roxmltree`.

Defines the `XmlDocument` and `XmlElement` traits as the common interface for
reading XML in the hick pipeline. The concrete implementation leaks source text
to extend lifetimes to `'static`, enabling zero-copy document storage. Entry
points are `parse_xml` and `try_parse_xml`.

Also includes a mini selector engine supporting `#id`, `//tag`, and
`//tag[@attr='value']` queries, plus `find_elements_in_documents` for searching
across multiple documents at once.

This is the XML parsing foundation for the pipeline; all code that reads raw
`.hick` or other XML files goes through these traits.
