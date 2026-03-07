# blue-iron-cube

**System of Record for Acme Drainage**

A french drain installation business operations platform. Tracks jobs, costs,
crew, inventory, and equipment to build a data-driven operation ready for
franchise standardization.

## Quick Start

```bash
cd backend
cargo run
```

The API runs at `http://localhost:3000`. Data is stored in `data/blue-iron-cube.db`.

## Domains


### Customers
- `GET /api/customers` — List customers
- `POST /api/customers` — Create customer
- `GET /api/customers/:id` — Get customer
- `PUT /api/customers/:id` — Update customer


### Jobs
- `GET /api/jobs` — List jobs (filterable by status, date range)
- `POST /api/jobs` — Create job
- `GET /api/jobs/:id` — Get job with all line items
- `PUT /api/jobs/:id` — Update job
- `POST /api/jobs/:id/line-items` — Add line item (material, labor, equipment, overhead)
- `GET /api/jobs/:id/costs` — Cost breakdown including processing fees
- `POST /api/jobs/:id/phase` — Mark a phase complete (assessment, excavation, pipe_install, backfill, restoration, walkthrough)
- `POST /api/jobs/:id/complete` — Mark fully complete
- `POST /api/jobs/:id/cancel` — Cancel or mark partially completed (with reason)


### Payments
- `POST /api/payments` — Record payment (cash, check, credit card, ACH, financing)
- `GET /api/payments/:id` — Get payment details
- `POST /api/refunds` — Issue a refund (partial or full)
- `GET /api/jobs/:id/payments` — Payment summary: balance due, fees absorbed, refunds
- `GET /api/payments/processing-fees/summary` — How much lost to credit card fees across all jobs


### Crew
- `GET /api/crew` — List crew members
- `POST /api/crew` — Add crew member
- `POST /api/crew/:id/time-entries` — Log hours against a job
- `GET /api/crew/:id/time-entries` — Time entry history


### Inventory
- `GET /api/inventory` — List materials with current stock
- `POST /api/inventory` — Add material type
- `POST /api/inventory/:id/receive` — Record stock received
- `POST /api/inventory/:id/use` — Record material used on a job
- `GET /api/inventory/low-stock` — Items below reorder point


### Fleet
- `GET /api/fleet/vehicles` — List vehicles
- `GET /api/fleet/equipment` — List equipment
- `POST /api/fleet/vehicles` — Add vehicle
- `POST /api/fleet/equipment` — Add equipment
- `POST /api/fleet/:id/maintenance` — Log maintenance
- `POST /api/fleet/:id/fuel` — Log fuel purchase
- `GET /api/fleet/:id/costs` — Total cost of ownership for an asset


### Overhead
- `GET /api/overhead/categories` — List overhead categories
- `POST /api/overhead/categories` — Add overhead category
- `POST /api/overhead/entries` — Record overhead expense
- `GET /api/overhead/recovery-rate` — Current overhead recovery rate
- `POST /api/overhead/allocate` — Allocate overhead to jobs in a period


### Estimating
- `POST /api/estimates` — Generate estimate for a new job
- `GET /api/estimates/:id` — Get a saved estimate
- `GET /api/estimates/model-stats` — Current regression model accuracy
- `POST /api/estimates/retrain` — Retrain model on latest job data


### CRM
- `GET /api/crm/leads` — List leads (filterable by status)
- `POST /api/crm/leads` — Create lead
- `GET /api/crm/leads/:id` — Get lead with customer details
- `PUT /api/crm/leads/:id` — Update lead status, mark won/lost
- `GET /api/crm/leads/pipeline` — Pipeline summary (count and value per stage)
- `POST /api/crm/interactions` — Log a customer interaction (call, email, text, site visit)
- `GET /api/crm/customers/:id/interactions` — Interaction history for a customer
- `GET /api/crm/follow-ups` — All pending follow-ups (sorted by due date)
- `POST /api/crm/follow-ups` — Schedule a follow-up
- `POST /api/crm/follow-ups/:id/complete` — Mark follow-up done
- `GET /api/crm/customers/:id/lifetime-value` — Total revenue, referrals, job history
- `GET /api/crm/conversion-rate` — Win/loss rate overall and by lead source


### Standard Operating Procedures
- `GET /api/sops` — List procedures
- `POST /api/sops` — Create procedure
- `GET /api/sops/:id` — Get procedure with steps
- `PUT /api/sops/:id` — Update procedure


## Architecture

This is a monolithic Rust backend with SQLite. Domains are organized as modules
with their own routes, models, and database tables. The data model is designed
so every dollar and every hour on a job is accounted for, enabling accurate
overhead recovery analysis and predictive estimating.

## License

Proprietary — Acme Drainage
