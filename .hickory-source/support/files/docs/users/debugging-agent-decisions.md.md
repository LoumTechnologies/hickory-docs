<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/users/debugging-agent-decisions.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `5b71e95b87b2e398da6e4409f5a1b3aa352c18fa6eed55caeef3f9aeb9b69278`.

<book:entry path="docs/users/debugging-agent-decisions.md">
# Revisit why an agent did something

Open the agent's session document and expand **Assumption audit**. Sessions are
ordinary local `.md` files; keep the session or use **Download full session
record** if you need to investigate it later.

Receipt chips with captured passages open a historical evidence reading. The
citing paragraph and literal quote comparisons appear beside the recorded
passage. An occurrence does not prove attribution, preserve negation, or support
a conclusion. References without captured passages still open the current file.

Search for a file, requirement, assumption or decision. The audit includes
abandoned branches and corrections. Each declaration shows the explanation,
evidence claimed, alternatives, affected work and what would disprove it.
**Supersedes** links a correction to the original, which stays in the record.
Use **Add an audit note or correction** to record your own finding.

Use **Show** to inspect other evidence:

- **Requested actions and outcomes** and **Work to inspect for missing
  assumptions** identify work whose explanation may be incomplete.
- **Historical source passages** show the bytes returned then, with a separate
  indication of whether today's source changed. **Compare with current source**
  opens today's file.
- **Executed verification** records actual Hickory checks and their revision.
  A model saying “tests passed” does not create this evidence.
- **Review decisions** show accepted and rejected document changes.
- **Adapter reported work** contains external agent reports, with the adapter's
  permissions and visibility limits.
- **Citation target checks** check availability of a current path and range.
  An available target does not mean it supports the conclusion.

A model declaration and your annotation remain declarations. The audit cannot
prove that it found every hidden assumption. Investigate omissions by comparing
the action record, source passages, requirements and alternatives.

Built-in document edits use **Review** by default in the app. **Auto-accept**
uses the same tools without a review click. Scripts follow the chosen executor;
external agent commands follow that agent's permissions. These controls do not
turn arbitrary commands into reviewed document edits.

For guarantee maintenance, ask the agent to record the original requirement,
assumptions, exclusions, independent witnesses and counterexamples. The skill
keeps LLM review, executed checks and human judgment separate. A changed
fingerprinted dependency makes earlier evidence historical, and unresolved
challenges remain reasons to investigate.
</book:entry>

</book:doc>
