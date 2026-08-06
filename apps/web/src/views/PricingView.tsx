import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { Plan, PlanPrice, PlansResponse } from "../api/types";

function priceFor(plan: Plan, interval: "month" | "year"): PlanPrice | null {
  return plan.prices.find((p) => p.interval === interval && !p.per_seat) ?? null;
}

function formatAmount(price: PlanPrice): string {
  const amount = (price.amount_cents / 100).toLocaleString(undefined, {
    style: "currency",
    currency: price.currency.toUpperCase(),
    maximumFractionDigits: price.amount_cents % 100 === 0 ? 0 : 2,
  });
  return `${amount}/${price.interval === "month" ? "mo" : "yr"}`;
}

// Prices come exclusively from GET /api/billing/plans — never hard-coded here.
export function PricingView() {
  const [data, setData] = useState<PlansResponse | null>(null);
  const [interval, setInterval] = useState<"month" | "year">("month");
  const [error, setError] = useState<string | null>(null);
  const [busyKey, setBusyKey] = useState<string | null>(null);

  useEffect(() => {
    api.plans().then(setData, (e) => setError(String(e.message ?? e)));
  }, []);

  const checkout = async (price: PlanPrice) => {
    setBusyKey(price.key);
    try {
      const { checkout_url } = await api.checkout(price.key);
      window.location.assign(checkout_url);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusyKey(null);
    }
  };

  return (
    <div className="pricing-page">
      <h1>Pricing</h1>
      <p className="muted">Verified documents, from local runs to Canopy microVMs.</p>
      {error && <p className="error">{error}</p>}
      {data === null ? (
        <p className="muted">Loading plans…</p>
      ) : (
        <>
          <div className="segmented interval-toggle" role="tablist">
            <button
              role="tab"
              aria-selected={interval === "month"}
              className={interval === "month" ? "on" : ""}
              onClick={() => setInterval("month")}
            >
              Monthly
            </button>
            <button
              role="tab"
              aria-selected={interval === "year"}
              className={interval === "year" ? "on" : ""}
              onClick={() => setInterval("year")}
            >
              Annual
            </button>
          </div>
          <div className="plan-grid">
            {data.plans.map((plan) => {
              const price = priceFor(plan, interval);
              const free = plan.prices.length === 0;
              return (
                <div key={plan.key} className={`plan-card${plan.highlight ? " highlight" : ""}`}>
                  <h2>{plan.name}</h2>
                  <p className="plan-price">{free ? "Free" : price ? formatAmount(price) : "—"}</p>
                  <p className="plan-desc">{plan.description}</p>
                  <ul>
                    {plan.features.map((f) => (
                      <li key={f}>{f}</li>
                    ))}
                  </ul>
                  {free ? (
                    <button className="btn" disabled>
                      Current plan
                    </button>
                  ) : (
                    <button
                      className={`btn${plan.highlight ? " btn-primary" : ""}`}
                      disabled={!price || busyKey === price.key}
                      onClick={() => price && void checkout(price)}
                    >
                      {busyKey === price?.key
                        ? "…"
                        : plan.trial_days
                          ? `Start ${plan.trial_days}-day trial`
                          : `Choose ${plan.name}`}
                    </button>
                  )}
                </div>
              );
            })}
          </div>
          {data.enterprise?.contact && (
            <p className="muted enterprise-note">
              Enterprise: {data.enterprise.description}{" "}
              <a href="mailto:nate@loumtechnologies.com">Contact us.</a>
            </p>
          )}
        </>
      )}
    </div>
  );
}
