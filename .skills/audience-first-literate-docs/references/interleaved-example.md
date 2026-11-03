<lit:doc xmlns:lit="http://www.hickorydocs.com/1.0">
# Refuse a request that exceeds the remaining quota

For maintainers adding a quota rule while preserving refusal behavior.

A caller requesting 3 units from a balance of 8 receives a remaining balance of
5. A negative request or a request above the available balance raises an error.

<lit:project />

## Check the request before calculating the result

Validate both bounds first. Without the lower-bound check, subtracting a negative
request would increase the balance. The function accepts integers by convention;
this example does not provide runtime type validation or persistent storage.

<lit:copy id="quota-validate">
def take(available: int, requested: int) -> int:
    if requested < 0 or requested > available:
        raise ValueError("request must be within the available quota")
</lit:copy>

Once the request has passed both checks, subtraction produces the remaining
balance. The function returns a value; it does not update stored state.

<lit:copy id="quota-subtract">
    return available - requested
</lit:copy>

## Demonstrate a valid request and both refusals

The first assertion checks a concrete result. The second checks negative requests
and requests above the balance, so removing either validation branch fails a case.
These tests do not establish concurrency or persistence guarantees.

<lit:copy id="quota-tests">
import unittest

from quota import take


class QuotaTests(unittest.TestCase):
    def test_valid_request_returns_remaining_quota(self):
        self.assertEqual(take(8, 3), 5)

    def test_invalid_requests_are_refused(self):
        for requested in (-1, 9):
            with self.subTest(requested=requested):
                with self.assertRaises(ValueError):
                    take(8, requested)
</lit:copy>

## Assemble the two working files

The implementation joins validation and subtraction in that order. Adjacent
paste tags avoid adding a blank line or changing the function's indentation.
The tests import the generated implementation; no separate code example is kept.

<lit:file path="quota.py"><lit:paste select="#quota-validate"/><lit:paste select="#quota-subtract"/></lit:file>
<lit:file path="test_quota.py"><lit:paste select="#quota-tests"/></lit:file>

After construction, run `python3 -m unittest -v test_quota` from the output
directory. This command is a procedure for the maintainer; this document contains
no execution cell and does not claim those tests ran merely because it was read.

</lit:doc>
