CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(username) BETWEEN 1 AND 60),
    password TEXT NOT NULL CHECK(password GLOB '$argon2*' AND length(password) >= 80),
    role TEXT NOT NULL DEFAULT 'user' CHECK(role IN ('admin', 'user')),
    active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0, 1)),
    must_change_password INTEGER NOT NULL DEFAULT 1 CHECK(must_change_password IN (0, 1)),
    theme_preference TEXT NOT NULL DEFAULT 'system' CHECK(theme_preference IN ('system', 'light', 'dark')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE teams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(name) BETWEEN 1 AND 60),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE team_members (
    team_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (team_id, user_id),
    FOREIGN KEY(team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE team_notification_settings (
    team_id INTEGER PRIMARY KEY REFERENCES teams(id) ON DELETE CASCADE,
    slack_webhook_url TEXT CHECK (slack_webhook_url IS NULL OR length(slack_webhook_url) BETWEEN 1 AND 2048),
    microsoft_teams_webhook_url TEXT CHECK (microsoft_teams_webhook_url IS NULL OR length(microsoft_teams_webhook_url) BETWEEN 1 AND 2048),
    telegram_bot_token TEXT CHECK (telegram_bot_token IS NULL OR length(telegram_bot_token) BETWEEN 1 AND 2048),
    telegram_chat_id TEXT CHECK (telegram_chat_id IS NULL OR length(telegram_chat_id) BETWEEN 1 AND 255),
    smtp_host TEXT CHECK (smtp_host IS NULL OR length(smtp_host) BETWEEN 1 AND 255),
    smtp_port INTEGER CHECK (smtp_port IS NULL OR smtp_port BETWEEN 1 AND 65535),
    smtp_security TEXT NOT NULL DEFAULT 'starttls' CHECK (smtp_security IN ('none', 'starttls', 'tls')),
    smtp_auth INTEGER NOT NULL DEFAULT 1 CHECK (smtp_auth IN (0, 1)),
    smtp_username TEXT CHECK (smtp_username IS NULL OR length(smtp_username) BETWEEN 1 AND 320),
    smtp_password TEXT CHECK (smtp_password IS NULL OR length(smtp_password) BETWEEN 1 AND 2048),
    smtp_from_email TEXT CHECK (smtp_from_email IS NULL OR length(smtp_from_email) BETWEEN 1 AND 320),
    smtp_to_email TEXT CHECK (smtp_to_email IS NULL OR length(smtp_to_email) BETWEEN 1 AND 320),
    notify_site_down INTEGER NOT NULL DEFAULT 1 CHECK (notify_site_down IN (0, 1)),
    notify_site_recovered INTEGER NOT NULL DEFAULT 1 CHECK (notify_site_recovered IN (0, 1)),
    notify_cert_expiring INTEGER NOT NULL DEFAULT 1 CHECK (notify_cert_expiring IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (telegram_bot_token IS NULL AND telegram_chat_id IS NULL)
        OR (telegram_bot_token IS NOT NULL AND telegram_chat_id IS NOT NULL)
    )
);

CREATE TABLE sites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 100),
    url TEXT NOT NULL CHECK(length(url) BETWEEN 10 AND 2000),
    expected_status INTEGER DEFAULT 200 CHECK(expected_status BETWEEN 100 AND 599),
    expected_text TEXT CHECK(expected_text IS NULL OR length(expected_text) BETWEEN 1 AND 500),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'up', 'down', 'blocked')),
    last_checked_at DATETIME,
    last_response_time_ms INTEGER,
    probe_interval_seconds INTEGER NOT NULL DEFAULT 60 CHECK(probe_interval_seconds BETWEEN 60 AND 3600),
    tls_allow_untrusted INTEGER NOT NULL DEFAULT 0 CHECK(tls_allow_untrusted IN (0, 1)),
    cert_status TEXT CHECK(cert_status IS NULL OR cert_status IN ('valid', 'expiring', 'critical', 'expired', 'invalid', 'none')),
    cert_expires_at DATETIME,
    cert_checked_at DATETIME,
    team_id INTEGER REFERENCES teams(id) ON DELETE RESTRICT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE outages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    http_status INTEGER CHECK(http_status IS NULL OR http_status BETWEEN 100 AND 599),
    expected_status INTEGER CHECK(expected_status IS NULL OR expected_status BETWEEN 100 AND 599),
    error_message TEXT CHECK(error_message IS NULL OR length(error_message) <= 500),
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at DATETIME
);

CREATE UNIQUE INDEX idx_one_open_outage ON outages(site_id) WHERE ended_at IS NULL;
CREATE INDEX idx_team_members_user_id ON team_members(user_id);
CREATE UNIQUE INDEX idx_sites_team_url ON sites(team_id, url);
CREATE UNIQUE INDEX idx_sites_url_no_team ON sites(url) WHERE team_id IS NULL;
