<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/SurfaceInfo.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6519bb14ac359e50936686b68b556927c5f184ccd3bc2bd1a2eeafcbea1c4968`.

<book:entry path="apps/web/src/components/SurfaceInfo.css">
.surface-info { flex-shrink: 0; min-width: 0; padding: .35rem .65rem; font-size: .78rem; color: var(--fg-muted); border-bottom: 1px solid var(--border); }
.surface-info > summary { display: flex; align-items: center; gap: .5rem; cursor: pointer; list-style: none; white-space: nowrap; }
.surface-info > summary::-webkit-details-marker { display: none; }
.surface-info > summary::before { content: "›"; flex-shrink: 0; }
.surface-info[open] > summary::before { transform: rotate(90deg); }
.surface-info strong { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; color: var(--fg); font-weight: 500; }
.surface-info > summary > span { flex-shrink: 0; }
.surface-info > p { margin: .5rem 0 0; overflow-wrap: anywhere; }
</book:entry>

</book:doc>
