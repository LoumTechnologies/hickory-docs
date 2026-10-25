<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/desktop/src-tauri/tauri.conf.json

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a306dc6a6169f3993f585a94dca3ab3f8dbda9bb2dd60f02aa496ba569c78754`.

<book:entry path="apps/desktop/src-tauri/tauri.conf.json">
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Hickory Docs",
  "version": "0.1.0",
  "identifier": "com.loumtechnologies.hickorydocs",
  "build": {
    "frontendDist": "../../web/dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm --prefix ../web run dev",
    "beforeBuildCommand": "npm --prefix ../web run build"
  },
  "app": {
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico",
      "icons/icon.png"
    ],
    "fileAssociations": [
      {
        "ext": [
          "md"
        ],
        "name": "Markdown document",
        "description": "Markdown document",
        "role": "Editor",
        "rank": "Default",
        "mimeType": "text/markdown",
        "contentTypes": [
          "net.daringfireball.markdown"
        ]
      }
    ],
    "windows": {
      "wix": {
        "fragmentPaths": [
          "windows/command-path.wxs"
        ],
        "componentRefs": [
          "HickoryUserPath"
        ]
      }
    }
  }
}
</book:entry>

</book:doc>
