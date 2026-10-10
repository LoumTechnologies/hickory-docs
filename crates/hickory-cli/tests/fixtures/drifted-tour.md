<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="text-tools-tour.md">
# A Verified Tour of Unix Text Tools

This document is executable. Every example below actually runs when the
document is built, and the outputs shown are the outputs produced. If a
future version of these tools changes behavior, `hick test` fails and
this document refuses to pretend otherwise.

## Setting the stage

We create a small data file that the rest of the document uses.

<hick:container name="shell" image="alpine:3.20" />

<hick:exec container="shell">
printf 'cherry,7\napple,3\nbanana,5\napple,9\n' > fruit.csv
cat fruit.csv
<hick:expect match="exact">cherry,7
apple,3
banana,5
apple,9
</hick:expect>
</hick:exec>

## Sorting

`sort` orders lines lexicographically by default; `-t, -k2 -n` sorts by the
numeric second field instead.

<hick:exec container="shell">
sort -t, -k2 -n fruit.csv
<hick:expect match="exact">apple,3
banana,5
cherry,7
apple,9
</hick:expect>
</hick:exec>

## Aggregating with awk

Summing the counts per fruit is one awk idiom every engineer ends up
needing:

<hick:exec container="shell">
awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' fruit.csv | sort
<hick:expect match="exact">apple 13
banana 5
cherry 7
</hick:expect>
</hick:exec>

## Verifying claims, not vibes

Documentation often claims "this is fast" or "this handles the edge case."
Here the claim *is* the test. An empty input produces no output — and the
document proves it:

<hick:exec container="shell">
printf '' | awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' | wc -l
<hick:expect match="exact">0
</hick:expect>
</hick:exec>
</hick:doc>
