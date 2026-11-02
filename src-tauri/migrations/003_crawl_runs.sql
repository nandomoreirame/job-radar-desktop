-- Migration 3: the crawl history behind the sidebar. `stats` is JSON from the pipeline.
CREATE TABLE crawl_runs (
  id INTEGER PRIMARY KEY,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  stats TEXT NOT NULL DEFAULT '{}'
);
