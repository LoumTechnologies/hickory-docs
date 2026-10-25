<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .claude/skills/persona-walkthrough/agents/openai.yaml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f8999b7bca81c2963a36b707fe868de7ab1eef5750223c554cc1a59fe31d391f`.

<book:entry path=".claude/skills/persona-walkthrough/agents/openai.yaml">
interface:
  display_name: "Persona Walkthrough"
  short_description: "Walk a persona through the whole funnel; report typed confusion"
  default_prompt: "Use $persona-walkthrough to load a persona from the library and walk it, strictly in character, through the entire funnel — social/discovery → website → learning the tool → decide → download → install → actually using the software → upsell/monetization trigger — surfacing where that specific human hesitates, doubts, or drops off. Bind the product at run time; drive live surfaces with $audit-feature-test-coverage's Playwright substrate or walk statically over code/docs/assets, and say which. At each stage ask what the persona is thinking, what latent capability they don't know about, what they should do next, and whether their paid boundary is hit correctly. Emit a typed confusion report (where·expected·got·did_instead·gave_up·severity·quote), append learnings to the persona log, and return the four-field upward contract (feasibility, observed reality, confusion, declared unknowables). Take no external action and mutate nothing; leave funnel instrumentation to $implement-growth-experiments and page construction to $landing-pages."
</book:entry>

</book:doc>
