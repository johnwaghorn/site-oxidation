CREATE TABLE sites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 100),
    url TEXT NOT NULL UNIQUE CHECK(length(url) BETWEEN 10 AND 2000),
    expected_status INTEGER DEFAULT 200 CHECK(expected_status BETWEEN 100 AND 599),
    expected_text TEXT CHECK(expected_text IS NULL OR length(expected_text) BETWEEN 1 AND 500),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'up', 'down')),
    last_checked_at DATETIME,
    last_response_time_ms INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE outages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    http_status INTEGER CHECK(http_status IS NULL OR http_status BETWEEN 100 AND 599),
    error_message TEXT CHECK(error_message IS NULL OR length(error_message) <= 500),
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at DATETIME
);

CREATE INDEX idx_outages_site_id ON outages(site_id);
