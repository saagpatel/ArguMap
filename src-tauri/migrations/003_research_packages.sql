CREATE TABLE IF NOT EXISTS research_packages (
  map_id            TEXT PRIMARY KEY REFERENCES maps(id) ON DELETE CASCADE,
  schema_version    TEXT NOT NULL,
  package_id        TEXT NOT NULL,
  revision_id       TEXT NOT NULL,
  canonical_package TEXT NOT NULL,
  adapter_losses    TEXT NOT NULL,
  updated_at        DATETIME DEFAULT CURRENT_TIMESTAMP
);
