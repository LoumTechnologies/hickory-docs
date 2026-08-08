---
skills:
  - plan-deploy-shared
---

# Configuration & Environment Architecture

## Rules (language- and provider-independent)

* **ALWAYS use strongly-typed configuration:** Deserialize and validate all environment variables into strict, strongly-typed configuration objects/structs at application startup.
* **NEVER use stringly-typed config:** Do not read raw environment strings directly within application logic. If a configuration is missing or malformed, the application must fail fast at boot.
* **ALWAYS expose environmental knobs:** Any parameter, URL, credential, timeout, or feature flag that could change across execution contexts (**Local Dev, CI System Tests, Review Apps, Staging, Production**) must be configurable via the environment schema. Never hardcode environment-specific overrides or assumptions.
* Environments such as production, staging, local dev environments, and review apps must have an identical set of environment variables configuring them, but with different values. The staging infrastructure must be cloud based and as similar as possible to production, but with smaller scale in order to be cheaper — never host staging on a developer machine (that is the local dev environment, not staging). Local dev environment should truly be local.
* **Staging and production share one canonical variable/secret name set** across
  their GitHub Environments — same names, environment-specific values, no
  per-environment name twins. The inventory (and the values to set in the GitHub
  UI) lives in an `ENVIRONMENTS.md` (`cloud/infra/ENVIRONMENTS.md` where the repo
  has a `cloud/` tree, otherwise `docs/operators/ENVIRONMENTS.md`), and a CI job
  (`ci.yml` → `env-parity`) fails the build if the two environments drift apart.
  Write the parity checker in the repo's primary backend language, per `just`.
* **Every `vars.*`/`secrets.*` reference lives inside an `environment:
  staging` or `environment: production` job — never a repo-level variable
  or secret.** A repo-level value is invisible to `env-parity`, bypasses the
  `production` environment's required-reviewer gate, and is reachable from
  both branches at once — exactly the blast radius the staging/production
  split exists to avoid. The parity checker fails the build if it finds a
  `vars.*`/`secrets.*` reference in a job with no `environment:` set.
* **One canonical name per value; the value differs, the name does not.** Never
  add a `staging_*` variable twin, and never hardcode an environment-specific
  literal in a workflow `env:` block (e.g. a domain, zone id, or OAuth client
  id) — read `vars.*` / `secrets.*` and let each GitHub Environment supply its
  own value under the shared name. The deploy target already distinguishes
  staging from production, so a value that differs only needs one variable. When
  you add a cross-environment name, record it in `ENVIRONMENTS.md`; a name
  legitimately in only one environment goes in the parity checker's allowlist.
  Otherwise the `env-parity` check fails.
* **Never name a GitHub Environment/repo variable or secret with a `GITHUB_`
  prefix.** GitHub reserves that prefix for its own built-in variables and
  rejects any `vars.*`/`secrets.*` with that name at creation time — the
  value silently stays unset, and workflows that read it fail with a
  confusing "missing required input" rather than a naming error. Use `GH_*`
  instead (e.g. `GH_OAUTH_CLIENT_ID`, matching `GH_OAUTH_CLIENT_SECRET`).
* **To add a variable, add a field to the service's typed config object** — never
  a new ad-hoc read at the point of use. Classify it as you add it:
  * a **secret**, or a value that differs across environments → **required**, no
    default, fail loud at boot when missing;
  * a **true constant** with one value everywhere → a single default on the
    field;
  * a **non-secret local-dev knob** → a default supplied by the dev-environment
    tooling, not by production code.
  Then set its value for staging and production through the matching GitHub
  Environment. An `APP_ENV=staging|production` marker should make the strict
  profile refuse to start on any missing required value.
* Always use Github Environments named production and staging to contain variables and secrets
* There should be orthogonality between staging and production environments in terms of secrets and variables provided as input via Github Environments; avoid defaults or logic that infers values for specific environments unless that logic can be applied equally to all environments.
* There must never be Github Actions secrets or variables that are not in a Github Environment.
* **Always** use sandboxes for any third-party integrations we provide if they are available for non-production environments. For example, if this software uses Stripe, the staging and local dev environments must use the Stripe sandbox.

## Stack-specific shapes

The rules above are what matter. The snippets below are **examples of the same
rule in different stacks** — use the one matching the repo you are in, and do
not carry another stack's idiom into it.

### Rust

Deserialize into a `Config` struct with `envconfig` (the portfolio's
`cloud/config` crate). Required fields have no `default`; a true constant gets a
single `#[envconfig(default = "…")]`; a non-secret local-dev knob goes in the
service's `port_zero_config::prepare(&[…])` dev-profile list. Services read
through that `Config`, never ad-hoc `std::env::var`. Environment values are set
in the relevant service's `docker-compose.*.yml`.

### NestJS / TypeScript

Validate the whole environment once at boot and inject typed slices — never
`configService.get<string>('FOO')` at a call site, which is stringly-typed and
silently yields `undefined` when unset.

```ts
// src/config/env.validation.ts — one schema for the whole process
class EnvVars {
  @IsString() @IsNotEmpty() DATABASE_URL!: string;   // required: no default
  @IsString() @MinLength(32) JWT_SECRET!: string;    // required: no default
  @IsString() @IsOptional() SENDGRID_API_KEY?: string; // optional integration
  @IsInt() @IsOptional() PORT?: number;
}

export function validate(raw: Record<string, unknown>): EnvVars {
  const parsed = plainToInstance(EnvVars, raw, { enableImplicitConversion: true });
  const errors = validateSync(parsed, { skipMissingProperties: false });
  if (errors.length) throw new Error(`Invalid environment:\n${errors.join('\n')}`);
  return parsed;
}
```

```ts
// src/app.module.ts — the app refuses to boot on a bad environment
ConfigModule.forRoot({ isGlobal: true, validate })
```

Group related values with `registerAs` and inject them as `ConfigType<typeof …>`
so consumers get a typed object rather than a string lookup. An optional
integration is modeled as an optional field whose absence the owning service
degrades on (see `third-party-integration-mocking`) — not as an untyped read.

### Vite / browser

`import.meta.env` is inlined at build time, so validate it in **one** module that
exports a typed, frozen object and throws at import time when a required value is
missing. No component reads `import.meta.env` directly. Remember that anything
reaching the browser bundle is public — never put a secret behind a `VITE_`
name.
