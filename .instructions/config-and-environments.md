# Configuration

This product has no deployment, so it has no environments: no staging, no
production, no GitHub Environments, no secret parity, no `env-parity` check.
The rules that survive are about reading configuration **from the user's
machine**, which is harder to get right than reading it from a deployment we
control — a bad value fails on someone else's laptop, where nobody can debug it
for them.

## Rules

* **ALWAYS use strongly-typed configuration.** Deserialize and validate every
  environment variable into a strict typed struct at startup. If a value is
  missing or malformed, fail at boot with a message naming the variable, what
  was wrong with it, and what a valid value looks like.
* **NEVER use stringly-typed config.** No ad-hoc `std::env::var` at the point
  of use. A new setting means a new field on the config struct.
* **Every knob that could reasonably differ between users is a variable.**
  Executor choice, cache location, editor integration paths, model/provider
  selection. Never hardcode an assumption about one machine.
* **Default to working with nothing set.** The tool must run on a clean machine
  with no environment at all. A required variable is a bug unless the feature
  genuinely cannot exist without it — and then it must be an *optional feature*
  that degrades, not a startup failure.
* **A missing credential degrades, never crashes.** No API key means the agent
  is unavailable and says so; it does not take down `hick up`. See
  `third-party-integration-mocking`.
* **Never require network access.** Every command except the agent must work
  fully offline, including on a machine that has never had internet.
* **Secrets belong to the user and stay on their machine.** Read them from the
  environment. Never write one to a file we create, never log one, never
  include one in a transcript, and never send one anywhere except to the
  provider the user chose.

## Naming

Environment variables are prefixed `HICKORY_` (`HICKORY_EXECUTOR`,
`HICKORY_PUBLIC_URL`), matching what already exists. Provider credentials keep
the names the provider ecosystem already uses (`ANTHROPIC_API_KEY`,
`OPENAI_API_KEY`) so a user's existing environment works without translation.

## Rust shape

Deserialize into a `Config` struct with `envconfig`. Required fields have no
default; a true constant gets a single `#[envconfig(default = "…")]`. Services
read through that `Config`, never ad-hoc `std::env::var`.
