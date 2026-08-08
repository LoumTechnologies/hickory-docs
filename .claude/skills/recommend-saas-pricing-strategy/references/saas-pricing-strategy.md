# SaaS Pricing Strategy Heuristics

Use these heuristics when evaluating a SaaS business model and recommending pricing strategy. They are distilled from Patrick McKenzie-style SaaS pricing advice and adapted for agent execution.

## Underpricing Pattern

- Technical founders often price from their own willingness to pay, which is usually far below business value.
- $9/month and $19/month plans are rarely enough for meaningful B2B SaaS unless distribution is exceptional.
- Raising prices can improve the whole business model because one higher-value customer may replace many low-value signups and reduce support and marketing burden.
- When a product creates meaningful business value, a $49/$99/$249 style grid may be more plausible than $9/$19/$49. Validate against the product's market, not as a universal template.

## Business Model Fit

- Match pricing to the buying motion. Self-serve plans need clear public packaging; sales-assisted and enterprise plans can support custom discovery, procurement, and security review.
- High-touch sales, implementation, compliance, or support usually require higher ACV than a cheap self-serve plan can support.
- Products tied to revenue, risk reduction, compliance, employee productivity, or customer operations can usually support higher prices than personal productivity tools.
- If customer acquisition is expensive, the pricing model must support payback. Low ARPA plus paid acquisition plus support burden is usually fragile.

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
- Use quotas to clarify fit and protect costs, but use customer value and buying context to justify price.

## Strategy Patterns

- Price ladder: Offer a serious entry plan, a clearly better default plan, and a high-value business or scale plan.
- Enterprise path: Add "Contact sales" or "Talk to us" when procurement, custom security, high volume, custom terms, or high consequence workflows are plausible.
- Annual anchor: Prefer annual plans or annual-first framing when cash flow, commitment, and activation make sense.
- Add-ons: Use add-ons for optional cost drivers or advanced capabilities that do not define the core buyer segment.
- Usage-based component: Use usage pricing when value and cost both scale with usage, but avoid making buyers forecast obscure units before purchase.

## One-Time And Perpetual Licences

These heuristics were written for recurring SaaS. Downloadable software sold as a perpetual licence differs in ways that change the answer:

- **The category sets the model, not preference.** Where every paid competitor is perpetual (desktop utilities, developer tools, trade software), a subscription is a positioning penalty. Where the category is subscription-native, perpetual signals hobby software.
- **Anchor on the avoided event, then sanity-check against paid competitors — never against free ones.** Free competitors do not set the price; they set the *conversion problem*. Being cheaper than free is impossible, so the differentiator has to carry the sale. If it cannot be shown in a screenshot, it usually cannot be charged for.
- **Price bands behave differently from SaaS.** Roughly: under $50 is a reflex purchase evaluated as an app; $200–$1000 is evaluated as equipment, with the buyer asking "does this work the way we work" rather than "is this cheap". Choosing the band is a positioning decision that precedes choosing the number. Pricing business software into the reflex band invites it to be judged as a toy.
- **Perpetual licence plus a time-boxed update window** is the standard shape: the licence works forever capped at a major version, and updates past the window need a renewal. Price the renewal at roughly 20–40% of the licence, and keep it genuinely optional.
- **Seats as Checkout quantity, not tiers.** Only split into separate plans when features genuinely differ. When the unit of value is a household or a site rather than a person, price that unit instead — charging per person for shared access to shared data prices the core benefit as an upsell.
- **Revenue is lumpy and there is no expansion revenue.** No renewals means no compounding, so acquisition cost has to be repaid by the *first* sale. That makes an unproven channel a much larger risk than it is for SaaS, where a mediocre channel can still pay back over a few years of retention. Say so when the channel is unproven.
- **Refund rate is the retention metric.** With no subscription to churn out of, a one-time purchase that goes unused converts to a refund. Adoption friction shows up as refunds, not cancellations.

## Output Shape

When advising, include:

- "Diagnosis": where the current business model likely leaks revenue or creates buyer confusion.
- "Buyer map": buyer segments, value received, willingness-to-pay signals, and acquisition motion.
- "Recommended grid": plan names, prices, buyer fit, value metric, included features, limits, annual framing, and enterprise path.
- "Migration": grandfathering, communication, and customer-success implications.
- "Validation": interviews, sales-assisted quotes, email-list offers, landing-page tests, pricing experiments, and instrumentation gaps.
