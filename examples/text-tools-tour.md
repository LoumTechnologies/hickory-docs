<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="text-tools-tour.md">
# A Verified Tour of Unix Text Tools

This document is executable. Every example below actually runs when the
document is built, and the outputs shown are the outputs produced. If a
future version of these tools changes behavior, `hick test` fails and
this document refuses to pretend otherwise.

## What runs where

Every cell below declares the container it runs in, and the declaration
renders as an environment card in the web app. The card names the
container, its image ref, and where commands actually execute: on the
local executor they run directly on the host and the image ref is
recorded provenance, not an enforced sandbox; on Cloud Canopy the image
ref maps to a Nix-defined cell image declared in `flake.nix`, and the
card shows the resolved store path (or warns when the ref is unmapped).

<hick:container name="shell" image="alpine:3.20" />

## Setting the stage

We create a small data file that the rest of the document uses.

<hick:exec container="shell">
printf 'cherry,7\napple,3\nbanana,5\napple,9\n' > fruit.csv
cat fruit.csv
<hick:expect match="exact">cherry,7
apple,3
banana,5
apple,9
</hick:expect>

<hick:ingested key="20f7f4ffe2e13efbbb0151ef9c31e951098e9b03b6032f1caea01d5288842c84" sha256="c671c0546676748666eff9dceeb796e3e75c9f633366aadc9c712165c92c414b" at="2026-09-03">
cherry,7
apple,3
banana,5
apple,9
</hick:ingested>
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

<hick:ingested key="7ed01f4d083b74dfea3e9f5cdb70a4554d216e6de3e6e1aa502daafe212973e1" sha256="92bf6daf65a6da1d76c8a54a02257f6498dcef1d590c40d27dec850e76b59167" at="2026-09-03">
apple,3
banana,5
cherry,7
apple,9
</hick:ingested>
</hick:exec>

## Aggregating with awk

Summing the counts per fruit is one awk idiom every engineer ends up
needing:

<hick:exec container="shell">
awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' fruit.csv | sort
<hick:expect match="exact">apple 12
banana 5
cherry 7
</hick:expect>

<hick:ingested key="d2c1c495c084d1da49cce7ca28081e1221767bfc8212504be58c995b832ea0ed" sha256="54f1c9c54973c847357c5b4d46e541e4feaffdd6c2b6a0ce553fccff4768a835" at="2026-09-03">
apple 12
banana 5
cherry 7
</hick:ingested>
</hick:exec>

## Verifying claims, not vibes

Documentation often claims "this is fast" or "this handles the edge case."
Here the claim *is* the test. An empty input produces no output — and the
document proves it.

The `tr -d ' '` is not decoration. GNU `wc -l` prints `0`, while the BSD
`wc` that ships with macOS right-aligns the count in an eight-column field
and prints `       0`. Both are correct; only one of them matches a literal
`0`. A document that runs on more than one machine has to say which of the
two it means, and stripping the padding is how this one says it — the
alternative is an expectation that passes on Linux and fails on a Mac for a
reason that has nothing to do with the claim being made.

<hick:exec container="shell">
printf '' | awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' | wc -l | tr -d ' '
<hick:expect match="exact">0
</hick:expect>

<hick:ingested key="2edd2fa0b2a2cbc09acdd34d59bc74003cad6800d64c31e59d772807878d449e" sha256="9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa" at="2026-09-03">
0
</hick:ingested>
</hick:exec>
</hick:doc>
