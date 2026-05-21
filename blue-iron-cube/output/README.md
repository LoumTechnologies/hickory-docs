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












## Architecture

This is a monolithic Rust backend with SQLite. Domains are organized as modules
with their own routes, models, and database tables. The data model is designed
so every dollar and every hour on a job is accounted for, enabling accurate
overhead recovery analysis and predictive estimating.

## License

Proprietary — Acme Drainage
