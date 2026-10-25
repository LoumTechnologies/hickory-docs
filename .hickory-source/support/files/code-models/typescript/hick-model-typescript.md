<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# code-models/typescript/hick-model-typescript

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8015b1220f16e08644d250c45a14d82fad077acfcc424b506fa637ce25b9fd11`.

<book:entry path="code-models/typescript/hick-model-typescript" executable="true">
#!/bin/sh
# Launcher: the server is an ES module, so it is `node server.js` rather than
# an executable of its own. Named for what `hick code-model` looks up.
exec node "$(dirname "$(readlink -f "$0")")/server.js" "$@"
</book:entry>

</book:doc>
