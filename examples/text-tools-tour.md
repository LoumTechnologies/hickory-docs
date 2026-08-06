
# A Verified Tour of Unix Text Tools

This document is executable. Every example below actually runs when the
document is built, and the outputs shown are the outputs produced. If a
future version of these tools changes behavior, `hickory check` fails and
this document refuses to pretend otherwise.

## Setting the stage

We create a small data file that the rest of the document uses.



$ printf 'cherry,7\napple,3\nbanana,5\napple,9\n' > fruit.csv
  cat fruit.csv
cherry,7
apple,3
banana,5
apple,9



## Sorting

`sort` orders lines lexicographically by default; `-t, -k2 -n` sorts by the
numeric second field instead.

$ sort -t, -k2 -n fruit.csv
apple,3
banana,5
cherry,7
apple,9



## Aggregating with awk

Summing the counts per fruit is one awk idiom every engineer ends up
needing:

$ awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' fruit.csv | sort
apple 12
banana 5
cherry 7



## Verifying claims, not vibes

Documentation often claims "this is fast" or "this handles the edge case."
Here the claim *is* the test. An empty input produces no output — and the
document proves it:

$ printf '' | awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' | wc -l
0


