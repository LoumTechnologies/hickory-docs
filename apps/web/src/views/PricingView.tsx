import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { Plan } from "../api/types";

function formatPrice(plan: Plan): string {
  if (plan.amount_cents === 0) return "Free";
  const amount = (plan.amount_cents / 100).toLocaleString(undefined, {
    style: "currency",
    currency: plan.currency.toUpperCase(),
  });
  return `${amount}/${plan.interval === "month" ? "mo" : "yr"}`;
}

// Prices come exclusively from GET /api/billing/plans — never hard-coded here.
export function PricingView() {
  const [plans, setPlans] = useState<Plan[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busyKey, setBusyKey] = useState<string | null>(null);

  useEffect(() => {
    api.plans().then(
      (res) => setPlans(res.plans),
      (e) => setError(String(e.message ?? e)),
    );
  }, []);

  const checkout = async (plan: Plan) => {
    if (plan.amount_cents === 0) return;
    setBusyKey(plan.key);
    try {
      const { checkout_url } = await api.checkout(plan.price_key);
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
      {plans === null ? (
        <p className="muted">Loading plans…</p>
      ) : (
        <div className="plan-grid">
          {plans.map((plan) => (
            <div key={plan.key} className={`plan-card${plan.highlight ? " highlight" : ""}`}>
              <h2>{plan.name}</h2>
              <p className="plan-price">{formatPrice(plan)}</p>
              <p className="plan-desc">{plan.description}</p>
              <ul>
                {plan.features.map((f) => (
                  <li key={f}>{f}</li>
                ))}
              </ul>
              <button
                className={`btn${plan.highlight ? " btn-primary" : ""}`}
                disabled={plan.amount_cents === 0 || busyKey === plan.key}
                onClick={() => void checkout(plan)}
              >
                {plan.amount_cents === 0
                  ? "Current plan"
                  : busyKey === plan.key
                    ? "…"
                    : `Choose ${plan.name}`}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
