<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bootstrap-ci.md">
# The Bootstrap, Reproduced

A statistical paper you can re-run. This document estimates a 95% bootstrap
confidence interval for the mean of a small sample, generates its own figure,
and verifies its own numbers. Nothing here is pasted in; every value is
computed when the document is built, from a fixed seed.

## The sample

<hick:container name="py" image="python:3.12" />

<hick:exec container="py">
python3 - <<'EOF'
import random
random.seed(42)
sample = [round(random.gauss(50, 12), 1) for _ in range(30)]
with open("sample.txt", "w") as f:
    f.write("\n".join(map(str, sample)))
print(f"n = {len(sample)}")
print(f"mean = {sum(sample)/len(sample):.2f}")
EOF
<hick:expect match="regex-lines">n = 30
mean = \d+\.\d\d
</hick:expect>

<hick:ingested key="d3d26ed8a14a2a28c7723155c35249b25888b7e1a82fdca13144c5befaac8b95" sha256="c94cdf926ba9fe8f7f84aa29918385385251a4f1c57fef4796a42c22e377cb73" at="2026-09-03">
n = 30
mean = 50.28
</hick:ingested>
</hick:exec>

## Resampling

We draw 10,000 bootstrap resamples and take the 2.5th and 97.5th percentiles
of the resampled means.

<hick:exec container="py">
python3 - <<'EOF'
import random
random.seed(7)
sample = [float(x) for x in open("sample.txt")]
n = len(sample)
means = sorted(
    sum(random.choices(sample, k=n)) / n
    for _ in range(10_000)
)
lo, hi = means[249], means[9749]
print(f"95% CI: [{lo:.2f}, {hi:.2f}]")
with open("ci.txt", "w") as f:
    f.write(f"{lo:.4f} {hi:.4f}")
EOF
<hick:expect match="regex-lines">95% CI: \[\d+\.\d\d, \d+\.\d\d\]
</hick:expect>

<hick:ingested key="c13e5fa236bd983bdf2a8b30cbebb38f25df358d432fa46ef05f7ea7ac779318" sha256="76e748f2fa593016452c7a8e8079286344160a6568d24f7e42c649c37b246b5c" at="2026-09-03">
95% CI: [47.33, 53.17]
</hick:ingested>
</hick:exec>

## The figure

The histogram below is an SVG generated from the bootstrap distribution —
regenerated on every build, so the figure can never drift from the data.

<hick:file path="bootstrap-histogram.svg">
<hick:exec container="py" show="output">
python3 - <<'EOF'
import random
random.seed(7)
sample = [float(x) for x in open("sample.txt")]
n = len(sample)
means = [sum(random.choices(sample, k=n)) / n for _ in range(10_000)]
lo_s, hi_s = open("ci.txt").read().split()
lo, hi = float(lo_s), float(hi_s)

bins = 40
mn, mx = min(means), max(means)
w = (mx - mn) / bins
counts = [0] * bins
for m in means:
    counts[min(int((m - mn) / w), bins - 1)] += 1
peak = max(counts)

W, H, PAD = 640, 320, 30
bars = []
for i, c in enumerate(counts):
    x = PAD + i * (W - 2 * PAD) / bins
    h = (H - 2 * PAD) * c / peak
    inside = lo <= mn + (i + 0.5) * w <= hi
    fill = "#4a7dbd" if inside else "#c9d4e3"
    bars.append(
        f'<rect x="{x:.1f}" y="{H - PAD - h:.1f}" '
        f'width="{(W - 2 * PAD) / bins - 1:.1f}" height="{h:.1f}" fill="{fill}"/>'
    )
print(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">')
print(f'<text x="{W/2}" y="18" text-anchor="middle" font-family="sans-serif" '
      f'font-size="13">Bootstrap distribution of the mean (shaded = 95% CI)</text>')
print("".join(bars))
print("</svg>")
EOF

<hick:ingested key="8bf5f89dafef3e99bf6385788b5dfcf99bac3c40acd1ef01e26ac434b22d2129" sha256="cee6ee9f3efcda1d86e5462125447e3deeadb1355090b79054ccce9c92c5e5e4" at="2026-09-03">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 320">
<text x="320.0" y="18" text-anchor="middle" font-family="sans-serif" font-size="13">Bootstrap distribution of the mean (shaded = 95% CI)</text>
<rect x="30.0" y="289.7" width="13.5" height="0.3" fill="#c9d4e3"/><rect x="44.5" y="289.0" width="13.5" height="1.0" fill="#c9d4e3"/><rect x="59.0" y="289.3" width="13.5" height="0.7" fill="#c9d4e3"/><rect x="73.5" y="287.4" width="13.5" height="2.6" fill="#c9d4e3"/><rect x="88.0" y="287.4" width="13.5" height="2.6" fill="#c9d4e3"/><rect x="102.5" y="286.4" width="13.5" height="3.6" fill="#c9d4e3"/><rect x="117.0" y="283.7" width="13.5" height="6.3" fill="#c9d4e3"/><rect x="131.5" y="279.5" width="13.5" height="10.5" fill="#c9d4e3"/><rect x="146.0" y="277.2" width="13.5" height="12.8" fill="#c9d4e3"/><rect x="160.5" y="264.7" width="13.5" height="25.3" fill="#c9d4e3"/><rect x="175.0" y="257.1" width="13.5" height="32.9" fill="#c9d4e3"/><rect x="189.5" y="234.1" width="13.5" height="55.9" fill="#4a7dbd"/><rect x="204.0" y="218.3" width="13.5" height="71.7" fill="#4a7dbd"/><rect x="218.5" y="193.2" width="13.5" height="96.8" fill="#4a7dbd"/><rect x="233.0" y="168.2" width="13.5" height="121.8" fill="#4a7dbd"/><rect x="247.5" y="138.9" width="13.5" height="151.1" fill="#4a7dbd"/><rect x="262.0" y="106.7" width="13.5" height="183.3" fill="#4a7dbd"/><rect x="276.5" y="89.2" width="13.5" height="200.8" fill="#4a7dbd"/><rect x="291.0" y="46.1" width="13.5" height="243.9" fill="#4a7dbd"/><rect x="305.5" y="50.7" width="13.5" height="239.3" fill="#4a7dbd"/><rect x="320.0" y="39.2" width="13.5" height="250.8" fill="#4a7dbd"/><rect x="334.5" y="30.0" width="13.5" height="260.0" fill="#4a7dbd"/><rect x="349.0" y="47.8" width="13.5" height="242.2" fill="#4a7dbd"/><rect x="363.5" y="55.3" width="13.5" height="234.7" fill="#4a7dbd"/><rect x="378.0" y="101.4" width="13.5" height="188.6" fill="#4a7dbd"/><rect x="392.5" y="118.9" width="13.5" height="171.1" fill="#4a7dbd"/><rect x="407.0" y="148.2" width="13.5" height="141.8" fill="#4a7dbd"/><rect x="421.5" y="188.3" width="13.5" height="101.7" fill="#4a7dbd"/><rect x="436.0" y="210.7" width="13.5" height="79.3" fill="#4a7dbd"/><rect x="450.5" y="233.1" width="13.5" height="56.9" fill="#4a7dbd"/><rect x="465.0" y="250.8" width="13.5" height="39.2" fill="#c9d4e3"/><rect x="479.5" y="264.3" width="13.5" height="25.7" fill="#c9d4e3"/><rect x="494.0" y="274.2" width="13.5" height="15.8" fill="#c9d4e3"/><rect x="508.5" y="281.1" width="13.5" height="8.9" fill="#c9d4e3"/><rect x="523.0" y="284.4" width="13.5" height="5.6" fill="#c9d4e3"/><rect x="537.5" y="287.7" width="13.5" height="2.3" fill="#c9d4e3"/><rect x="552.0" y="289.3" width="13.5" height="0.7" fill="#c9d4e3"/><rect x="566.5" y="288.7" width="13.5" height="1.3" fill="#c9d4e3"/><rect x="581.0" y="289.3" width="13.5" height="0.7" fill="#c9d4e3"/><rect x="595.5" y="289.3" width="13.5" height="0.7" fill="#c9d4e3"/>
</svg>
</hick:ingested>
</hick:exec>
</hick:file>

## Why this matters

Most published figures cannot be regenerated from their paper. This one is
regenerated on every run of `hick run`, and `hick test` fails the
build if the computed interval stops matching the prose above. The document
is the analysis.
</hick:doc>
