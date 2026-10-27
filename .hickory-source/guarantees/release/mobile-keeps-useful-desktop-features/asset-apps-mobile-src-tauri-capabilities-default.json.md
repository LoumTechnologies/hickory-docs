<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Limit native permissions to the mobile window

Limit native permissions to the mobile window. This literal construction preserves build configuration exactly.

<mobile:entry path="apps/mobile/src-tauri/capabilities/default.json">
{
  "identifier": "mobile",
  "description": "The local mobile document window",
  "windows": [
    "main"
  ],
  "permissions": [
    "core:default"
  ]
}
</mobile:entry>
</mobile:doc>
