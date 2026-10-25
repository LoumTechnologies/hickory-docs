<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/coverage/client.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `76764d13d277f8f4c84c98fdc2e98db07b50a34760019756d4b3d037d81013f4`.

<book:entry path="apps/web/src/coverage/client.ts">
import { request } from "../api/client";
export const coverageCall = <T,>(body: Record<string, unknown>) => request<T>("POST", "/api/tests/coverage", body);
</book:entry>

</book:doc>
