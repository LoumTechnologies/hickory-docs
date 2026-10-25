<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/src/hickory_agent_worker/__main__.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Python agent code crosses two explicit boundaries](../../../../../contract.md)

Captured SHA-256: `571eeaa7c030022c2f1b2ee9d84ac7ec6cd558a4bee6c45c6c6523a580b8dd13`.

<book:entry path="apps/agent/src/hickory_agent_worker/__main__.py">
"""Private confined worker entry point. Use hickory-agent-dev for development."""
from .hosted import main

if __name__ == "__main__":
    main()
</book:entry>

</book:doc>
