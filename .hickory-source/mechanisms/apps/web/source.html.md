<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Serve the public source entry

For engineers maintaining the public source browser.

The static site has its own source-reading entry. Keep it separate from the homepage and desktop HTML so a deep link loads only the read-only source browser.

<storycode:copy id="public-source-apps-web-source-html">
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="color-scheme" content="light dark"><title>Read Hickory's source · Hickory Docs</title></head><body><div id="root"></div><script type="module" src="./src/embed/source.tsx"></script></body></html>
</storycode:copy>

<storycode:file path="apps/web/source.html"><storycode:paste select="#public-source-apps-web-source-html"/></storycode:file>
</storycode:doc>
