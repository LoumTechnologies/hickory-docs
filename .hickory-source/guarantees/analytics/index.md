<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Analytics

For: Maintainers evaluating landing-page behavior and visitor privacy.

Check whether the page and event still exist before treating historical analytics claims as current.

- [A Visit Is Reported Exactly Once Per Page Load](landing-events-fire-once/contract.md) — retained-requirement.
- [Declared Identity Is Recorded Separately From Revealed Interest, And Both](landing-declared-vs-revealed/contract.md) — historical.
- [Driving The Home-Page Demo Is Reported As Revealed Interest](landing-demo-engagement/contract.md) — retained-requirement.
- [Landing Attribution Credits The First Touch, And Survives The Hash Router](landing-attribution-is-first-touch/contract.md) — retained-requirement.
- [Opening An Interest Section Is Recorded, With Dwell On Close](landing-interest-vector/contract.md) — historical.
- [The Landing Beacon Forwards Only Declared Events With Scalar Properties](landing-beacon-is-narrow/contract.md) — historical.

<book:include file="landing-attribution-is-first-touch/contract.md" />
<book:include file="landing-beacon-is-narrow/contract.md" />
<book:include file="landing-declared-vs-revealed/contract.md" />
<book:include file="landing-demo-engagement/contract.md" />
<book:include file="landing-events-fire-once/contract.md" />
<book:include file="landing-interest-vector/contract.md" />

</book:doc>
