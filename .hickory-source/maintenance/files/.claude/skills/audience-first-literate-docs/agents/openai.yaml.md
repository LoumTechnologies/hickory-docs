<guidance:doc xmlns:guidance="http://www.hickorydocs.com/1.0">
# Maintain .claude/skills/audience-first-literate-docs/agents/openai.yaml

For maintainers updating the repository’s agent guidance.

This literal entry preserves the skill refresh committed in `efbe277f`. Keeping the
installed guidance in the declared source makes regeneration reproduce the same
instructions. Embedded Markdown and Hickory examples remain literal payload.

<guidance:entry path=".claude/skills/audience-first-literate-docs/agents/openai.yaml">
interface:
  display_name: "Audience First Literate Docs"
  short_description: "Index shared code through audience-specific views"
  default_prompt: "Use $audience-first-literate-docs to index source code through audience-specific literate views, adding reusable readings for likely recurring work and checking that each view preserves its backing code."
</guidance:entry>
</guidance:doc>
