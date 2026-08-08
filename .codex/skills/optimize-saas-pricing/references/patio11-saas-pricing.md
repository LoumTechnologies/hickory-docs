# Patio11 SaaS Pricing Heuristics

Use these heuristics when designing SaaS pricing and pricing experiments. They are distilled from Patrick McKenzie-style SaaS pricing advice and adapted for agent execution in a codebase.

## Underpricing Pattern

- Technical founders often price from their own willingness to pay, which is usually far below business value.
- $9/month and $19/month plans are rarely enough for meaningful B2B SaaS unless distribution is exceptional.
- Raising prices can improve the whole business model because one higher-value customer may replace many low-value signups and reduce support/marketing burden.
- When a product creates meaningful business value, a $49/$99/$249 style grid may be more plausible than $9/$19/$49. Validate against the product's market, not as a universal template.

## Segment By Value Received

- Identify who gets the most value, not only who signs up most often.
- Agencies, teams, departments, regulated companies, and businesses with payroll often have very different willingness to pay than freelancers or prosumers.
- Anchor price to a business event or outcome buyers already value: one missed appointment, one recovered lead, one employee-hour saved, one avoided outage, one closed deal, one avoided compliance problem.
- Keep early adopters on old plans when repricing unless there is a strong reason not to. Treat grandfathering as a cheap marketing and trust expense.

## Plan Names And Self-Segmentation

- Buyers frequently self-select by identity and organizational context, not by precise quota math.
- Plan names matter. A Fortune 500 employee may avoid expensing a plan named Hobbyist or Small Business even if the features fit.
- Prefer names that make target buyers comfortable: Starter, Team, Business, Growth, Scale, Enterprise, Agency, Practice, Clinic, Studio, etc. Choose names for the actual market.
- Avoid fanciful names when they obscure who the plan is for.

## Pricing Is Not Mainly Quota Math

- Customers often cannot forecast usage for abstract units before they use the product.
- Do not assume buyers optimize around cents-per-unit. They often anchor on plan position, plan name, risk, and perceived professionalism.
- The second-cheapest plan often captures many purchases, while the highest plan can produce most revenue.
- Test feature placement and plan framing, not only raw price.

## Experiment Ideas

- Price ladder test: show higher prices to a new-user cohort and compare revenue per visitor/account, not only signup rate.
- Feature-placement test: advertise a feature in tier X vs tier X+1 while backend entitlements remain generous enough to avoid customer harm.
- Quota-generosity test: keep prices constant and shift displayed quotas upward to see whether plan mix changes.
- Plan-name test: keep features/prices constant and test buyer-identity names against generic names.
- Annual-anchor test: test annual-first presentation or stronger annual discount framing.
- Enterprise-path test: add "Contact sales" or "Talk to us" for high-value customers and track qualified conversations.
- Channel/landing-page test: send different paid acquisition campaigns or partner pages to dedicated pricing/packaging variants.
- Email-list offer test: randomly split a selected list segment and send different offers or prices for the same plan.

## Statistical Planning

- Primary metrics should usually be revenue per visitor, revenue per account, paid conversion, expansion, or qualified sales conversations.
- Guardrails can include refund rate, support tickets, chargebacks, cancellation reason, activation, time to value, and retained revenue.
- Use account-level randomization for account products. Use visitor-level randomization only when anonymous browsing is the unit being tested.
- For low traffic, use large enough changes to matter. Tiny price differences are unlikely to be learnable.
- If there is no baseline instrumentation, first ship analytics and run a baseline period before claiming an experiment result.
- Prefer a predeclared decision rule: ship variant B if revenue per exposed account is higher and guardrails do not degrade materially after N exposed accounts or T weeks.
- Treat email-list tests as directional unless randomization, deliverability, consent, and conversion attribution are reliable.

## Email List Plan

- Build or verify consent capture in onboarding, trial signup, checkout, content downloads, and newsletter forms.
- Store source, consent timestamp, product segment, current plan, trial status, and lifecycle stage where possible.
- Segment offers by lifecycle: prospects, active trials, expired trials, free users, low-tier customers, churned customers, and high-intent leads.
- Use a holdout group when the list is large enough. For small lists, split a narrow segment and track replies plus purchases.
- Include clear offer terms, expiration, and a support path. Avoid training customers to wait for discounts by making tests occasional and targeted.

## Landing Page Plan

- Use dedicated landing pages for market segments or channels when the value story differs.
- Match the headline and proof to the buyer's business outcome, not to generic product capabilities.
- Keep pricing visible when price is part of qualification. Hide or qualify enterprise pricing when sales discovery is necessary.
- Track page view, pricing view, checkout start, trial start, purchase, sales contact, and retained activation.
- Keep pages honest: do not promise unavailable features. If testing copy before building a feature, make the CTA a waitlist, interview, or concierge offer.

## Output Shape

When advising, include:

- "Diagnosis": where current pricing likely leaks revenue.
- "Recommended grid": plan names, prices, buyer fit, value metric, included features, limits, and enterprise path.
- "Experiment": hypothesis, variants, audience, split, metrics, runtime, decision rule, and support policy.
- "Instrumentation": events, properties, revenue joins, dashboards, and tests needed.
- "Implementation": exact code areas to change when working in a repo.
