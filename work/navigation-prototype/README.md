# Spatial navigation prototype

Open `preview.html` in a browser. It is a standalone experiment with an agent above Files in the center, linked documents on either side, and Hickory-style provenance brackets. Compare conversation parallax, spatial zoom, and accordion movement; scroll the conversation, pin documents to a side, or follow a sample stdout event to its print statement.

`spatial-navigation.html` is the editable visualization source; `preview.html` is its browser-ready snapshot. This prototype uses sample content and simulated log mappings, and does not connect to the app or a running agent.

With the web project's Playwright dependency and Chromium installed, run from the repository root:

```sh
node work/navigation-prototype/check.cjs
```

The check exercises brackets, pinning, conversation navigation, log-to-source tracing, zoom, movement styles, and a narrow viewport. Set `PROTOTYPE_SCREENSHOTS=1` to also capture desktop and mobile screenshots.
