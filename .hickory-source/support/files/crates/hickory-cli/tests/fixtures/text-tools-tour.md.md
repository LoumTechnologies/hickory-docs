<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/fixtures/text-tools-tour.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `51fd03b73121bbbcf0997f1e5ed5f0b5cf7e5096a5b2ca2778fa5329e59f185a`.

<book:entry path="crates/hickory-cli/tests/fixtures/text-tools-tour.md">

# A Verified Tour of Unix Text Tools

This document is executable. Every example below actually runs when the
document is built, and the outputs shown are the outputs produced. If a
future version of these tools changes behavior, `hick test` fails and
this document refuses to pretend otherwise.

## Setting the stage

We create a small data file that the rest of the document uses.



$ printf 'cherry,7\napple,3\nbanana,5\napple,9\n' > fruit.csv
  cat fruit.csv
[never run]


## Sorting

`sort` orders lines lexicographically by default; `-t, -k2 -n` sorts by the
numeric second field instead.

$ sort -t, -k2 -n fruit.csv
[never run]


## Aggregating with awk

Summing the counts per fruit is one awk idiom every engineer ends up
needing:

$ awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' fruit.csv | sort
[never run]


## Verifying claims, not vibes

Documentation often claims "this is fast" or "this handles the edge case."
Here the claim *is* the test. An empty input produces no output — and the
document proves it:

$ printf '' | awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' | wc -l
[never run]

</book:entry>

</book:doc>
