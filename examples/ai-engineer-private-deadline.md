# Private retry-budget practice

Synthetic source and inputs, with an intentionally faulty allowance calculation.
The requirement is a single 100ms total budget: preserve the short and empty
cases and predict the allowance before the second attempt.

Use this source through a startup-bound private coach. Ordinary Run has ordinary
output disclosure; it does not establish this exercise's private boundary.

<hick:container name="c" image="python" />
<hick:volume name="code" input="." />
<hick:file path="deadline.py">
def retry(attempts, budget):
    elapsed = 0
    for duration in attempts:
        allowance = budget
        elapsed += min(duration, allowance)
    return elapsed
import json
print(json.dumps({"long": retry([80, 80], 100), "short": retry([10, 20], 100), "empty": retry([], 100)}))
</hick:file>
<hick:exec container="c" mount="code:/work">
python3 work/deadline.py
<hick:capture at="deadline.py:5" of="allowance" max="2" />
<hick:expect>{"long": 100, "short": 30, "empty": 0}
</hick:expect>
</hick:exec>
