import { useState } from "react";
import { api } from "../api/client";
import generatedPlans from "../api/generated/plans.json";
import { config } from "../config";
import type { Plan, PlanPrice, PlansResponse } from "../api/types";
import { InstallCommand } from "../components/InstallCommand";

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

// Prices are never hard-coded here. They come from `plans.json` through the
// server's own projection (`plans::plans_response`), generated at `just
// codegen` into api/generated/plans.json — so a static site shows the same
// pricing the hosted endpoint would have served, with no request to make.
const PLANS = generatedPlans as PlansResponse;

export function PricingView() {
  const [interval, setInterval] = useState<"month" | "year">("month");
  const [error, setError] = useState<string | null>(null);
  const [busyKey, setBusyKey] = useState<string | null>(null);
  const data = PLANS;

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
      {!config.hosted && (
        <div className="banner banner-warn pricing-note">
          <p>
            <strong>These plans are for the hosted workspace.</strong> The tool
            itself — running documents, verification, the agent, and live
            sessions you host from your own machine — is free and needs no
            account.
          </p>
          <InstallCommand />
        </div>
      )}
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
                  {!config.hosted ? null : free ? (
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
    </div>
  );
}
