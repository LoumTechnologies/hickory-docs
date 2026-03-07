CREATE TABLE IF NOT EXISTS vehicles (
    id TEXT PRIMARY KEY,
    vehicle_type TEXT NOT NULL,
    year INTEGER NOT NULL,
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    license_plate TEXT,
    vin TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    monthly_payment REAL,
    insurance_monthly REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS equipment (
    id TEXT PRIMARY KEY,
    equipment_type TEXT NOT NULL,
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    serial_number TEXT,
    owned_or_rented TEXT NOT NULL DEFAULT 'owned',
    daily_rate REAL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS maintenance_records (
    id TEXT PRIMARY KEY,
    asset_type TEXT NOT NULL CHECK(asset_type IN ('vehicle', 'equipment')),
    asset_id TEXT NOT NULL,
    date TEXT NOT NULL,
    maintenance_type TEXT NOT NULL,
    cost REAL NOT NULL,
    vendor TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS fuel_logs (
    id TEXT PRIMARY KEY,
    vehicle_id TEXT NOT NULL REFERENCES vehicles(id),
    date TEXT NOT NULL,
    gallons REAL NOT NULL,
    cost_per_gallon REAL NOT NULL,
    total_cost REAL NOT NULL,
    odometer INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_maintenance_asset ON maintenance_records(asset_id);
CREATE INDEX idx_fuel_vehicle ON fuel_logs(vehicle_id);
