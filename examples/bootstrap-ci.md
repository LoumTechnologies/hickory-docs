
# The Bootstrap, Reproduced

A statistical paper you can re-run. This document estimates a 95% bootstrap
confidence interval for the mean of a small sample, generates its own figure,
and verifies its own numbers. Nothing here is pasted in; every value is
computed when the document is built, from a fixed seed.

## The sample



$ python3 - <<'EOF'
  import random
  random.seed(42)
  sample = [round(random.gauss(50, 12), 1) for _ in range(30)]
  with open("sample.txt", "w") as f:
      f.write("\n".join(map(str, sample)))
  print(f"n = {len(sample)}")
  print(f"mean = {sum(sample)/len(sample):.2f}")
  EOF
n = 30
mean = 50.28



## Resampling

We draw 10,000 bootstrap resamples and take the 2.5th and 97.5th percentiles
of the resampled means.

$ python3 - <<'EOF'
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
95% CI: [47.33, 53.17]



## The figure

The histogram below is an SVG generated from the bootstrap distribution —
regenerated on every build, so the figure can never drift from the data.


### `bootstrap-histogram.svg`

```

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 320">
<text x="320.0" y="18" text-anchor="middle" font-family="sans-serif" font-size="13">Bootstrap distribution of the mean (shaded = 95% CI)</text>
<rect x="30.0" y="289.7" width="13.5" height="0.3" fill="#c9d4e3"/><rect x="44.5" y="289.0" width="13.5" height="1.0" fill="#c9d4e3"/><rect x="59.0" y="289.3" width="13.5" height="0.7" fill="#c9d4e3"/><rect x="73.5" y="287.4" width="13.5" height="2.6" fill="#c9d4e3"/><rect x="88.0" y="287.4" width="13.5" height="2.6" fill="#c9d4e3"/><rect x="102.5" y="286.4" width="13.5" height="3.6" fill="#c9d4e3"/><rect x="117.0" y="283.7" width="13.5" height="6.3" fill="#c9d4e3"/><rect x="131.5" y="279.5" width="13.5" height="10.5" fill="#c9d4e3"/><rect x="146.0" y="277.2" width="13.5" height="12.8" fill="#c9d4e3"/><rect x="160.5" y="264.7" width="13.5" height="25.3" fill="#c9d4e3"/><rect x="175.0" y="257.1" width="13.5" height="32.9" fill="#c9d4e3"/><rect x="189.5" y="234.1" width="13.5" height="55.9" fill="#4a7dbd"/><rect x="204.0" y="218.3" width="13.5" height="71.7" fill="#4a7dbd"/><rect x="218.5" y="193.2" width="13.5" height="96.8" fill="#4a7dbd"/><rect x="233.0" y="168.2" width="13.5" height="121.8" fill="#4a7dbd"/><rect x="247.5" y="138.9" width="13.5" height="151.1" fill="#4a7dbd"/><rect x="262.0" y="106.7" width="13.5" height="183.3" fill="#4a7dbd"/><rect x="276.5" y="89.2" width="13.5" height="200.8" fill="#4a7dbd"/><rect x="291.0" y="46.1" width="13.5" height="243.9" fill="#4a7dbd"/><rect x="305.5" y="50.7" width="13.5" height="239.3" fill="#4a7dbd"/><rect x="320.0" y="39.2" width="13.5" height="250.8" fill="#4a7dbd"/><rect x="334.5" y="30.0" width="13.5" height="260.0" fill="#4a7dbd"/><rect x="349.0" y="47.8" width="13.5" height="242.2" fill="#4a7dbd"/><rect x="363.5" y="55.3" width="13.5" height="234.7" fill="#4a7dbd"/><rect x="378.0" y="101.4" width="13.5" height="188.6" fill="#4a7dbd"/><rect x="392.5" y="118.9" width="13.5" height="171.1" fill="#4a7dbd"/><rect x="407.0" y="148.2" width="13.5" height="141.8" fill="#4a7dbd"/><rect x="421.5" y="188.3" width="13.5" height="101.7" fill="#4a7dbd"/><rect x="436.0" y="210.7" width="13.5" height="79.3" fill="#4a7dbd"/><rect x="450.5" y="233.1" width="13.5" height="56.9" fill="#4a7dbd"/><rect x="465.0" y="250.8" width="13.5" height="39.2" fill="#c9d4e3"/><rect x="479.5" y="264.3" width="13.5" height="25.7" fill="#c9d4e3"/><rect x="494.0" y="274.2" width="13.5" height="15.8" fill="#c9d4e3"/><rect x="508.5" y="281.1" width="13.5" height="8.9" fill="#c9d4e3"/><rect x="523.0" y="284.4" width="13.5" height="5.6" fill="#c9d4e3"/><rect x="537.5" y="287.7" width="13.5" height="2.3" fill="#c9d4e3"/><rect x="552.0" y="289.3" width="13.5" height="0.7" fill="#c9d4e3"/><rect x="566.5" y="288.7" width="13.5" height="1.3" fill="#c9d4e3"/><rect x="581.0" y="289.3" width="13.5" height="0.7" fill="#c9d4e3"/><rect x="595.5" y="289.3" width="13.5" height="0.7" fill="#c9d4e3"/>
</svg>


```


## Why this matters

Most published figures cannot be regenerated from their paper. This one is
regenerated on every run of `hickory run`, and `hickory check` fails the
build if the computed interval stops matching the prose above. The document
is the analysis.
