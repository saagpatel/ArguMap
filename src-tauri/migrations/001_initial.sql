-- ArguMap Studio — initial schema

CREATE TABLE IF NOT EXISTS maps (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL DEFAULT 'Untitled Map',
  description TEXT,
  created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_maps_updated ON maps(updated_at DESC);

CREATE TABLE IF NOT EXISTS nodes (
  id          TEXT PRIMARY KEY,
  map_id      TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
  node_type   TEXT NOT NULL CHECK(node_type IN ('claim','evidence','rebuttal','counter_rebuttal')),
  content     TEXT NOT NULL DEFAULT '',
  source      TEXT,
  x           REAL NOT NULL DEFAULT 100,
  y           REAL NOT NULL DEFAULT 100,
  width       REAL NOT NULL DEFAULT 220,
  height      REAL NOT NULL DEFAULT 80,
  created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_nodes_map ON nodes(map_id);

CREATE TABLE IF NOT EXISTS edges (
  id             TEXT PRIMARY KEY,
  map_id         TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
  source_node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  target_node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  edge_type      TEXT NOT NULL CHECK(edge_type IN ('supports','rebuts','qualifies','depends_on')),
  label          TEXT,
  UNIQUE(source_node_id, target_node_id, edge_type)
);
CREATE INDEX IF NOT EXISTS idx_edges_map ON edges(map_id);
