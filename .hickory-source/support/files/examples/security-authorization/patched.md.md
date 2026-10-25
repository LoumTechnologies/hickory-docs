<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/security-authorization/patched.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `b60f7ed89deae86c9026c7dfe1535e68f3f39d4cf8fb37fd186876f33db12102`.

<book:entry path="examples/security-authorization/patched.md">
# Synthetic cross-tenant authorization: patched

R-AUTH-1: A tenant must not access a record owned by another tenant.
R-LEGIT-1: A tenant can access its own existing record; missing records stay denied.

All tenants and records below are synthetic. There is no network, customer data,
credential, service endpoint or real target. This fixture checks one authorization
predicate, not authentication, storage isolation, concurrency or other security properties.

<hick:container name="c" image="python" />
<hick:volume name="code" input="." />
<hick:file path="authorization.py">
records = {"record-a": "tenant-a", "record-b": "tenant-b"}
def authorize(tenant, record_id):
    allowed = int(record_id in records and records[record_id] == tenant)
    return allowed

cross_tenant = authorize("tenant-a", "record-b")
assert authorize("tenant-a", "record-a") == 1
assert authorize("tenant-b", "record-b") == 1
assert authorize("tenant-a", "missing") == 0
print(cross_tenant)
</hick:file>
<hick:exec container="c" mount="code:/work">
python3 work/authorization.py
<hick:capture at="authorization.py:4" of="allowed" max="1" />
<hick:expect>0
</hick:expect>
</hick:exec>

The selected debugger question is `allowed` at the first `authorize` return,
for tenant-a requesting tenant-b's record. Seal both output and debugger
predictions as zero before managed execution. Keep that prediction and both
requirements unchanged when applying the tenant-ownership patch.

`hick:capture` uses an ordinary execution followed by a separate scratch debugger
execution of these source bytes. Captured stops are recorded evidence when viewed
later; viewing them does not start a debugger. Missing capture evidence cannot pass.
These source files are templates, not selected publication artifacts or evidence
that an application strict mode is available.
</book:entry>

</book:doc>
