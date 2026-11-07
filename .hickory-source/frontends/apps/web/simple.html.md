<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Open a separate browser entry

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Mount a separate root so none of the existing workspace shell is imported. Vite serves this entry beside the original editor.

<ui:copy id="simple-5bfb0491c8-0">
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Hickory Simple</title></head>
<body><div id="root"></div><script type="module" src="/src/simple/main.tsx"></script></body></html>
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/simple.html"><ui:paste select="#simple-5bfb0491c8-0"/></ui:file>

</ui:doc>
