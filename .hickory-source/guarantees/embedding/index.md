<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Embedding

For: Host developers embedding the document editor without a native execution service.

Identify the host responsibilities before assuming a browser view can execute native tools.

- [A local save is revision checked](a-local-save-is-revision-checked/contract.md) — retained-requirement.
- [A viewer snapshot keeps recorded results historical](a-viewer-snapshot-keeps-recorded-results-historical/contract.md) — retained-requirement.
- [An embed has no implicit host](an-embed-has-no-implicit-host/contract.md) — retained-requirement.
- [An iframe validates its host](an-iframe-validates-its-host/contract.md) — retained-requirement.
- [Literal files use native bytes](literal-files-use-native-bytes/contract.md) — retained-requirement.
- [Offline edits keep their sync base](offline-edits-keep-their-sync-base/contract.md) — retained-requirement.
- [Published executable results identify reviewed inputs](publication-is-bound-to-reviewed-inputs/contract.md) — retained-requirement.
- [S3 publishes one manifest](s3-publishes-one-manifest/contract.md) — retained-requirement.

<book:include file="a-local-save-is-revision-checked/contract.md" />
<book:include file="a-viewer-snapshot-keeps-recorded-results-historical/contract.md" />
<book:include file="an-embed-has-no-implicit-host/contract.md" />
<book:include file="an-iframe-validates-its-host/contract.md" />
<book:include file="literal-files-use-native-bytes/contract.md" />
<book:include file="offline-edits-keep-their-sync-base/contract.md" />
<book:include file="publication-is-bound-to-reviewed-inputs/contract.md" />
<book:include file="s3-publishes-one-manifest/contract.md" />

</book:doc>
