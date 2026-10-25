<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/desktop/src-tauri/capabilities/default.json

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ef4f81e18e75967f0636bfc3e6c2699259f80d9cc9e9273e0165d2b01890e73b`.

<book:entry path="apps/desktop/src-tauri/capabilities/default.json">
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability for the Hickory Docs window",
  "windows": [
    "main"
  ],
  "platforms": [
    "linux",
    "macOS",
    "windows"
  ],
  "permissions": [
    "core:default",
    "dialog:default"
  ]
}
</book:entry>

</book:doc>
