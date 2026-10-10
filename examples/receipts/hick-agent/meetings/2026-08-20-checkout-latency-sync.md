---
date: 2026-08-20
attendees: [Sam, Nate, Priya]
source: 2026-08-20-checkout-latency-sync.vtt
source-format: vtt
source-sha256: 3c9dca9b9ccf707d6e5ecb70a33e557d4c06781d5cb94a61097a2efa105a170b
---

# Checkout latency sync

<hick:transcript id="transcript" format="vtt">
WEBVTT

00:00:04.000 --> 00:00:12.000
<v Sam>Checkout p95 latency has been bad since the August 12th deploy. I'm fairly sure the connection pool is exhausting at peak — we're at ten connections and the queue waits are all over the logs.

00:00:13.000 --> 00:00:19.000
<v Nate>What's the target again? I want to be able to say a number in the channel.

00:00:20.000 --> 00:00:26.000
<v Sam>The SLO is p95 under 300 milliseconds for checkout. We're nowhere near that right now.

00:00:27.000 --> 00:00:38.000
<v Priya>I have the hourly latency export for August 10th through the 21st in the analytics bucket. I can compute before and after for whatever window you pick. One warning: the 18th has the load test in it, ten to noon, so anything that averages over the 18th will be wrong unless you exclude that window.

00:00:39.000 --> 00:00:48.000
<v Sam>Proposal: raise the pool from ten to forty connections and put a two second acquire timeout on it so a stuck connection fails fast instead of backing everything up.

00:00:49.000 --> 00:00:55.000
<v Nate>Is forty safe for Postgres at our size?

00:00:56.000 --> 00:01:03.000
<v Sam>Yes, the instance is configured for two hundred and we have three app replicas. Forty each is a hundred twenty, which is inside that.

00:01:04.000 --> 00:01:13.000
<v Nate>OK. Decision: ship the pool change behind a config flag on Friday, Priya confirms with the numbers after it's been live for a day, and I'll post a summary in #eng with the before and after.

00:01:14.000 --> 00:01:18.000
<v Priya>Works for me. I'll exclude the load-test window and say so in the numbers.

00:01:19.000 --> 00:01:24.000
<v Sam>I'll have the fix up today. Action items: Sam writes the fix, Priya runs the numbers, Nate posts.
</hick:transcript>

## Summary

<hick:transform provider="openrouter" model="anthropic/claude-sonnet-4.5" select="#transcript" instruct="Summarize this meeting for someone who missed it, in at most five sentences: what was decided, what is blocked, and on whom. Plain prose, no bullet points." from="dc92819e">
Checkout p95 latency has been above the 300 millisecond SLO since the August 12th deploy, likely due to connection pool exhaustion at the current ten-connection limit. The team decided to raise the pool to forty connections per replica (well within the Postgres limit of two hundred) and add a two-second acquire timeout, shipping the change behind a config flag on Friday. Priya will compute before-and-after latency metrics using data from the analytics bucket, excluding the load test window on August 18th from ten to noon. Sam is writing the fix today, Priya will run the analysis after the change has been live for a day, and Nate will post a summary with the results in the #eng channel.
</hick:transform>

## Action items

<hick:transform provider="openrouter" model="anthropic/claude-sonnet-4.5" select="#transcript" instruct="Extract every action item as a markdown checklist, one line each, naming who owns it. Output only the list." from="58af386a">
- Sam: Write the fix to raise the connection pool from ten to forty connections with a two second acquire timeout behind a config flag
- Priya: Run the latency numbers comparing before and after the fix goes live, excluding the August 18th load test window (10am-noon)
- Nate: Post a summary in #eng with the before and after latency results
</hick:transform>
