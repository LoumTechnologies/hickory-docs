# Hickory Docs Requirements: Replicating tmplit's Feature System

This document analyzes tmplit's requirements for managing multi-variant codebases, specifically what a literate programming tool like Hickory Docs would need to support to enable AI/LLM coding agents to generate similar codebases from a single source.

## Executive Summary

tmplit manages **template repositories** where different combinations of optional features produce different codebases. It uses **git branches** to represent each feature combination, with changes cascading through a **directed acyclic graph (DAG)** based on feature subset relationships.

For Hickory Docs to replicate this, it needs:
1. **Conditional code generation** based on active features
2. **Feature dependency constraints** (feature A requires feature B)
3. **Text substitution with case variants** (project names across PascalCase, snake_case, etc.)
4. **Conditional file inclusion/exclusion**
5. **A way to express the feature→content relationship** that AI agents can understand and manipulate

---

## 1. Feature System

### 1.1 Feature Definition

tmplit defines features as named, optional capabilities with descriptions:

```toml
[features]
auth = { description = "Authentication and authorization" }
billing = { description = "Stripe billing integration" }
analytics = { description = "Usage analytics and dashboards" }
```

**Hickory Docs requirement:**
- Define named features with descriptions
- Features are boolean toggles (present or absent)
- AI agents need to understand what each feature adds to the codebase

### 1.2 Feature Dependencies (`requires`)

Features can depend on other features:

```toml
[features]
auth = { description = "Authentication" }
billing = { description = "Billing" }
premium-billing = { description = "Premium billing tiers", requires = ["auth", "billing"] }
```

**Constraints:**
- If feature X requires features [A, B], then X can only be enabled when both A and B are also enabled
- Dependency graph must be **acyclic** (no circular requires chains)
- A feature with requirements never appears "standalone" — its minimal configuration includes all required features

**Hickory Docs requirement:**
- Express feature dependencies
- Validate that selected feature sets satisfy all requirements
- When a feature is selected, automatically include its dependencies

### 1.3 Valid Feature Combinations

tmplit generates all valid combinations of features (the power set, minus invalid combinations where requirements aren't satisfied):

| Features defined | Valid combinations (excluding empty set) |
|-----------------|------------------------------------------|
| `[auth, billing]` | `{auth}`, `{billing}`, `{auth, billing}` |
| `[auth, billing, premium]` where premium requires auth+billing | `{auth}`, `{billing}`, `{auth, billing}`, `{auth, billing, premium}` |

**Invalid combinations are rejected:**
- `{premium}` alone (missing auth and billing)
- `{auth, premium}` (missing billing)

**Hickory Docs requirement:**
- Enumerate all valid feature combinations
- Reject/warn about invalid combinations
- AI agents should be able to query "what features can I add given current selection?"

---

## 2. Conditional Content Generation

### 2.1 Feature-Conditional Code Blocks

Code that only exists when certain features are enabled:

```
// Shared code (always present)
class App {
    init() {
        // core initialization
    }

    // ONLY when auth feature is enabled:
    #if(auth)
    func authenticate() { ... }
    #endif

    // ONLY when billing feature is enabled:
    #if(billing)
    func processPayment() { ... }
    #endif

    // ONLY when BOTH auth AND billing are enabled:
    #if(auth && billing)
    func purchaseWithAuth() { ... }
    #endif
}
```

**Hickory Docs requirement:**
- Conditional blocks based on feature predicates
- Support AND (`&&`), OR (`||`), NOT (`!`) operators
- Nested conditionals
- Clean output (no artifacts from the conditional system in generated code)

### 2.2 Feature-Conditional Files

Entire files that only exist for certain feature combinations:

```
src/
  core/           # always present
    app.rs
  auth/           # only when auth feature enabled
    login.rs
    session.rs
  billing/        # only when billing feature enabled
    stripe.rs
    invoices.rs
  premium/        # only when premium feature enabled (requires auth+billing)
    tiers.rs
```

**Hickory Docs requirement:**
- Conditionally include/exclude entire files
- Conditionally include/exclude entire directories
- AI agents need to understand which files belong to which features

### 2.3 Feature-Conditional Dependencies

Package dependencies that vary by feature:

```toml
# Always included
[dependencies]
serde = "1.0"

# Only when auth feature enabled
#if(auth)
[dependencies]
jsonwebtoken = "8.0"
#endif

# Only when billing feature enabled
#if(billing)
[dependencies]
stripe-rust = "0.20"
#endif
```

**Hickory Docs requirement:**
- Conditional sections in configuration files (Cargo.toml, package.json, etc.)
- Merge/combine conditional sections intelligently

---

## 3. Text Substitution System

### 3.1 Basic Pattern Substitution

Replace placeholder patterns with consumer-provided values:

```toml
[substitutions]
project_name = { pattern = "FavoriteApp", description = "Project name" }
bundle_id = { pattern = "com.example.favorite", description = "iOS bundle identifier" }
```

When consumer provides `project_name = "MyProject"`:
- All occurrences of `FavoriteApp` in code → `MyProject`
- All occurrences in file paths → renames `FavoriteApp.swift` to `MyProject.swift`

**Hickory Docs requirement:**
- Define placeholder patterns
- Replace in file contents
- Replace in file/directory names
- Longer patterns matched first (avoid partial replacements)

### 3.2 Case Variant Generation

When `variants = true`, automatically generate all case forms:

```toml
[substitutions]
project_name = { pattern = "FavoriteApp", description = "Project name", variants = true }
```

From pattern `FavoriteApp` and value `MyProject`, generate:

| Convention | Pattern | Replacement |
|-----------|---------|-------------|
| PascalCase | `FavoriteApp` | `MyProject` |
| camelCase | `favoriteApp` | `myProject` |
| snake_case | `favorite_app` | `my_project` |
| SCREAMING_SNAKE_CASE | `FAVORITE_APP` | `MY_PROJECT` |
| kebab-case | `favorite-app` | `my-project` |
| Title Case | `Favorite App` | `My Project` |
| lowercase | `favoriteapp` | `myproject` |
| UPPERCASE | `FAVORITEAPP` | `MYPROJECT` |

**Hickory Docs requirement:**
- Auto-generate case variants from a single pattern/value
- Support multi-word patterns (word boundaries detected from case)
- AI agents should use consistent naming across case conventions

### 3.3 Domain Name Variants

When pattern looks like a domain name, generate domain-specific variants:

```toml
bundle_id = { pattern = "com.exampleorg.favoriteapp", description = "Bundle ID", variants = true }
```

From pattern `com.exampleorg.favoriteapp` and value `com.mycompany.myapp`:

| Convention | Pattern | Replacement |
|-----------|---------|-------------|
| reverse-domain | `com.exampleorg.favoriteapp` | `com.mycompany.myapp` |
| forward-domain | `favoriteapp.exampleorg.com` | `myapp.mycompany.com` |
| https:// | `https://exampleorg.com` | `https://mycompany.com` |
| https://www. | `https://www.exampleorg.com` | `https://www.mycompany.com` |
| http:// | `http://exampleorg.com` | `http://mycompany.com` |
| http://www. | `http://www.exampleorg.com` | `http://www.mycompany.com` |

**Hickory Docs requirement:**
- Detect domain patterns (known TLDs: com, org, net, io, dev, co, edu, gov, app, me)
- Generate forward/reverse domain and URL variants
- Combine with case variants for the non-TLD segments

### 3.4 Substitution Validation

Validate consumer-provided values:

```toml
[substitutions]
bundle_id = {
    pattern = "com.example.favorite",
    description = "Bundle identifier",
    validation = "^[a-z][a-z0-9_]*(\\.[a-z][a-z0-9_]*)+$",
    validation_error = "Must be a reverse-domain name (e.g., com.mycompany.myapp)"
}
```

**Hickory Docs requirement:**
- Regex validation for substitution values
- Custom error messages
- Default values

---

## 4. File Exclusion System

### 4.1 Template-Only Files

Files that exist in the template but should not appear in consumer projects:

```toml
exclude = [
    "TEMPLATE_README.md",     # Template documentation
    "docs/internal",          # Internal developer notes
    "*.template-only",        # Any file with this extension
    ".github/workflows/template-ci.yml"  # Template-specific CI
]
```

**Hickory Docs requirement:**
- Glob patterns for exclusion
- Recursive directory exclusion
- Exclusions applied after generation (file exists in source, removed from output)

---

## 5. The Feature→Branch→Content Relationship

### 5.1 How tmplit Models This

tmplit uses **git branches** where each branch represents a feature combination:

```
main (core)           — no features, shared code only
├── auth              — {auth}
│   └── auth+billing  — {auth, billing}
├── billing           — {billing}
│   └── auth+billing  — {auth, billing}  [same branch, two parents]
└── analytics         — {analytics}
    ├── analytics+auth          — {analytics, auth}
    ├── analytics+billing       — {analytics, billing}
    └── analytics+auth+billing  — {analytics, auth, billing}
```

**Key insight:** The feature combination defines the branch, and the branch contains exactly the code for that combination.

### 5.2 What Hickory Docs Needs Instead

Since Hickory Docs is a **literate programming tool** (single source → multiple outputs), it needs a different model:

**Single source with conditional markers:**
```markdown
# App.swift

This file defines the main application class.

```swift
class App {
    func start() {
        // Core startup logic
    }

    <<auth-methods>>
    <<billing-methods>>
    <<analytics-methods>>
}
```

## Auth Methods
#if(auth)

```swift chunk: auth-methods
func authenticate(user: String, password: String) -> Bool {
    // authentication logic
}

func logout() {
    // logout logic
}
```

#endif

## Billing Methods
#if(billing)

```swift chunk: billing-methods
func processPayment(amount: Decimal) -> PaymentResult {
    // billing logic
}
```

#endif
```

**Requirements for this model:**
1. **Conditional chunk inclusion** — chunks defined within `#if` blocks
2. **Chunk references that resolve conditionally** — `<<auth-methods>>` expands to nothing if auth is disabled
3. **Clean output** — no `#if` markers in generated code
4. **Multiple output targets** — generate different file sets for different feature combinations

---

## 6. AI Agent Requirements

### 6.1 What AI Agents Need to Understand

For an AI agent to work with Hickory Docs the way it would work with tmplit:

1. **Feature discovery**: "What features are available? What does each do?"
2. **Feature constraints**: "If I want X, what else do I need?"
3. **Content location**: "Where is the code for feature X?"
4. **Modification scope**: "If I change this, which feature combinations are affected?"
5. **Consistent patterns**: "What naming convention is used? What substitution patterns exist?"

### 6.2 Metadata AI Agents Need

```yaml
# Hickory Docs should expose this to AI agents
features:
  auth:
    description: "Authentication and authorization"
    requires: []
    files:
      - src/auth/login.rs
      - src/auth/session.rs
    chunks:
      - auth-methods
      - auth-middleware
      - auth-config

  billing:
    description: "Stripe billing integration"
    requires: []
    files:
      - src/billing/stripe.rs
    chunks:
      - billing-methods
      - billing-webhooks

  premium:
    description: "Premium billing tiers"
    requires: [auth, billing]
    files:
      - src/premium/tiers.rs
    chunks:
      - premium-tier-logic

substitutions:
  project_name:
    pattern: "FavoriteApp"
    variants: true
    description: "Project name"

valid_feature_sets:
  - []  # core only
  - [auth]
  - [billing]
  - [auth, billing]
  - [auth, billing, premium]
```

### 6.3 Commands AI Agents Need

| tmplit command | Hickory Docs equivalent |
|---------------|------------------------|
| `tmplit feature list` | List all features with descriptions |
| `tmplit feature add X` | Add new feature definition to source |
| `tmplit cascade` | Regenerate all output variants |
| `tmplit check` | Validate feature constraints, substitution patterns |
| `tmplit use --features auth,billing` | Generate output for specific feature set |

---

## 7. The Merge/Cascade Problem

### 7.1 Why tmplit Uses Git Merges

When code is changed on a "parent" branch (e.g., `core` or `auth`), the change must propagate to all "child" branches (e.g., `auth+billing`). tmplit uses git merges for this:

```
Change on core → merge into auth → merge into auth+billing
              → merge into billing → merge into auth+billing (already done)
              → merge into analytics → merge into analytics+auth → ...
```

The DAG ensures changes flow "downstream" (from fewer features to more features).

### 7.2 How Hickory Docs Avoids This

With conditional generation from a single source, there's **no merge problem**:

1. Edit the source document
2. Regenerate all outputs
3. Each output reflects the change (if the change affects that feature combination)

**The key advantage:** No merge conflicts. The source is the single truth.

### 7.3 What Hickory Docs Must Handle Instead

- **Chunk ordering**: When multiple features add methods to a class, what order?
- **Conditional imports**: If auth adds `import AuthKit` and billing adds `import StripeKit`, both must appear when both features are enabled
- **Configuration merging**: Feature-specific config entries must combine correctly
- **Test organization**: Tests for each feature, plus integration tests when features combine

---

## 8. Summary: Hickory Docs Feature Checklist

### Must Have (Core)

- [ ] Named features with descriptions
- [ ] Feature dependencies (`requires`)
- [ ] Validation that feature sets satisfy constraints
- [ ] Conditional code blocks (`#if(feature)`)
- [ ] Conditional file inclusion
- [ ] Boolean operators in conditions (`&&`, `||`, `!`)
- [ ] Text substitution in content and paths
- [ ] Case variant generation (PascalCase, snake_case, etc.)
- [ ] File/directory exclusion patterns
- [ ] Clean output (no conditional markers in generated code)
- [ ] Generate output for any valid feature combination

### Should Have (AI Agent Support)

- [ ] Machine-readable feature metadata (for AI discovery)
- [ ] Query: "What files/chunks belong to feature X?"
- [ ] Query: "What features affect file Y?"
- [ ] Query: "Is feature set [A, B, C] valid?"
- [ ] Substitution pattern documentation in metadata
- [ ] Validation command (check consistency)

### Nice to Have (Advanced)

- [ ] Domain name variant generation
- [ ] Regex validation for substitution values
- [ ] Default values for substitutions
- [ ] Chunk ordering controls (when features combine)
- [ ] Import deduplication/merging
- [ ] Configuration file section merging
- [ ] Diff between feature combinations ("what does adding X change?")

---

## 9. Example: Mapping tmplit to Hickory Docs

### tmplit structure (branches)

```
core/
  src/app.rs        # shared

auth/
  src/app.rs        # shared + auth additions
  src/auth/login.rs # auth-only file

billing/
  src/app.rs        # shared + billing additions
  src/billing/pay.rs # billing-only file

auth+billing/
  src/app.rs        # shared + auth + billing additions
  src/auth/login.rs # from auth
  src/billing/pay.rs # from billing
```

### Hickory Docs structure (single source)

```markdown
# app.rs.lit

```rust file: src/app.rs
use crate::config::Config;
<<auth-imports>>
<<billing-imports>>

pub struct App {
    config: Config,
    <<auth-fields>>
    <<billing-fields>>
}

impl App {
    pub fn new() -> Self { ... }

    <<auth-methods>>
    <<billing-methods>>
}
```

<<auth-chunks>>
<<billing-chunks>>

# Auth Feature
#if(auth)

```rust chunk: auth-imports
use crate::auth::{Session, Authenticator};
```

```rust chunk: auth-fields
    session: Option<Session>,
```

```rust chunk: auth-methods
    pub fn login(&mut self, creds: Credentials) -> Result<()> { ... }
    pub fn logout(&mut self) { ... }
```

```rust chunk: auth-chunks
```

#endif

# Billing Feature
#if(billing)

```rust chunk: billing-imports
use crate::billing::{PaymentProcessor, Invoice};
```

```rust chunk: billing-fields
    payment_processor: PaymentProcessor,
```

```rust chunk: billing-methods
    pub fn charge(&self, amount: Money) -> Result<Invoice> { ... }
```

```rust chunk: billing-chunks
```

#endif
```

### Output for feature set `{auth, billing}`

```rust
// src/app.rs
use crate::config::Config;
use crate::auth::{Session, Authenticator};
use crate::billing::{PaymentProcessor, Invoice};

pub struct App {
    config: Config,
    session: Option<Session>,
    payment_processor: PaymentProcessor,
}

impl App {
    pub fn new() -> Self { ... }

    pub fn login(&mut self, creds: Credentials) -> Result<()> { ... }
    pub fn logout(&mut self) { ... }
    pub fn charge(&self, amount: Money) -> Result<Invoice> { ... }
}
```

---

## 10. Questions for Hickory Docs Design

1. **Syntax**: What's the conditional syntax? `#if(feature)` / `#endif`? `{{#if feature}}` / `{{/if}}`? Something else?

2. **Chunk references**: How do undefined/conditional chunks resolve? Empty string? Error? Configurable?

3. **Feature metadata**: Where is feature metadata defined? Front matter? Separate config file?

4. **Multi-file output**: How does one literate source produce multiple output files?

5. **Substitution timing**: Are substitutions applied before or after conditional resolution?

6. **AI integration**: How does an AI agent query the feature graph? Parse the source? API?

7. **Validation**: How are constraint violations reported? Compile-time errors? Warnings?

8. **Incremental generation**: Can it regenerate just affected outputs when source changes?
