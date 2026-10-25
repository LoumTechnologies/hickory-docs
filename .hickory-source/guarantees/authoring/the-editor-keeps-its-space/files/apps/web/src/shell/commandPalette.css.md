<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/commandPalette.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The editor keeps its space](../../../../../contract.md)

Captured SHA-256: `7edb7372322bee777072972a1f2c82275defbf834fa90b26630e9feb7493faeb`.

<book:entry path="apps/web/src/shell/commandPalette.css">
.command-palette[hidden] { display: none; }
.command-palette { position: fixed; inset: 0; z-index: 500; display: flex; justify-content: center; align-items: flex-start; padding-top: min(12vh, 100px); background: rgba(0, 0, 0, .18); }
.command-bar--overlay { width: min(38rem, calc(100vw - 24px)); padding: 8px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-raised); box-shadow: var(--shadow); }
.command-bar--overlay .command-bar__input { box-sizing: border-box; text-align: left; }
.command-bar--overlay .command-bar__panel { left: 0; right: 0; }
</book:entry>

</book:doc>
