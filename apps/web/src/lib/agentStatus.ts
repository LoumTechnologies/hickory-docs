import type { AcpState } from "../api/acp";

export interface QuotaWindow { label: string; used: number; resetsAt: number }
export interface AccountUsage { windows: QuotaWindow[]; resets: number | null; checkedAt: number; error?: string }

/** Reject expired or malformed windows; unknown is never zero used. */
export function codexUsage(raw: unknown, now = Date.now()): AccountUsage {
  const data = raw as { rateLimits?: { primary?: unknown; secondary?: unknown }; rateLimitsByLimitId?: Record<string, { primary?: unknown; secondary?: unknown }>; rateLimitResetCredits?: { availableCount?: unknown } } | null;
  const buckets = data?.rateLimitsByLimitId;
  const limits = buckets?.codex ?? data?.rateLimits;
  const windows = [limits?.primary, limits?.secondary].flatMap(value => {
    const v = value as { usedPercent?: unknown; windowDurationMins?: unknown; resetsAt?: unknown } | null;
    if (!v || typeof v.usedPercent !== "number" || !Number.isFinite(v.usedPercent) || v.usedPercent < 0 || v.usedPercent > 100 || typeof v.resetsAt !== "number" || !Number.isFinite(v.resetsAt) || v.resetsAt * 1000 <= now || typeof v.windowDurationMins !== "number" || v.windowDurationMins <= 0) return [];
    const minutes = v.windowDurationMins;
    return [{ label: minutes === 10080 ? "Week" : minutes === 300 ? "5h" : `${minutes}m`, used: v.usedPercent, resetsAt: v.resetsAt }];
  });
  const count = data?.rateLimitResetCredits?.availableCount;
  return { windows, resets: typeof count === "number" && Number.isSafeInteger(count) && count >= 0 ? count : null, checkedAt: now };
}

/** Read reported choices by category/id, never guess effort from a model name. */
export function agentIdentity(backend: string, provider: string, model: string, state: AcpState | null | undefined, name?: string): string {
  if (backend === "python") return ["Hickory agent", provider || "Provider not reported", model || "Model not reported"].join(" · ");
  const options = state?.configOptions ?? [];
  const choice = (kinds: string[]) => options.find(option => kinds.includes(option.category ?? "") || kinds.includes(option.id));
  const modelChoice = choice(["model"]);
  const effort = choice(["thought_level", "reasoning_effort", "effort"]);
  const speed = choice(["speed", "service_tier", "fast_mode"]);
  const label = (option: typeof modelChoice) => {
    const values = option?.options?.flatMap(item => "value" in item ? [item] : item.options);
    return values?.find(item => item.value === option?.currentValue)?.name ?? String(option?.currentValue);
  };
  return [`${name ?? backend} via ACP`, modelChoice ? label(modelChoice) : state?.models?.currentModelId || "Model not reported",
    effort && label(effort), speed && (speed.type === "boolean" ? speed.currentValue ? "Fast" : "Default speed" : label(speed))].filter(Boolean).join(" · ");
}

// DeepSeek's published schedule, checked 2026-11-04. Beijing has no DST.
// https://api-docs.deepseek.com/zh-cn/quick_start/pricing/
// Holiday ranges: State Council notice 国办发明电〔2025〕7号.
const HOLIDAYS_2026 = [["01-01", "01-03"], ["02-15", "02-23"], ["04-04", "04-06"], ["05-01", "05-05"], ["06-19", "06-21"], ["09-25", "09-27"], ["10-01", "10-07"]];
function peakAt(time: number): boolean | null {
  const beijing = new Date(time + 8 * 3600000);
  if (beijing.getUTCFullYear() !== 2026) return null;
  const day = beijing.getUTCDay(), date = beijing.toISOString().slice(5, 10);
  if (day === 0 || day === 6 || HOLIDAYS_2026.some(([start, end]) => date >= start && date <= end)) return false;
  const hour = beijing.getUTCHours();
  return hour >= 9 && hour < 12 || hour >= 14 && hour < 18;
}
export function deepseekWindow(now = Date.now()): { peak: boolean; changesAt: number } | null {
  if (!Number.isFinite(now)) return null;
  const peak = peakAt(now);
  if (peak === null) return null;
  // Every change is on an hour boundary. Search through a full holiday week.
  for (let next = Math.floor(now / 3600000) * 3600000 + 3600000; next < now + 16 * 86400000; next += 3600000) {
    const state = peakAt(next);
    if (state === null) return null;
    if (state !== peak) return { peak, changesAt: next };
  }
  return null;
}
export function timeLeft(until: number, now: number): string {
  const minutes = Math.max(0, Math.ceil((until - now) / 60000));
  return minutes >= 60 ? `${Math.floor(minutes / 60)}h ${minutes % 60}m` : `${minutes}m`;
}
