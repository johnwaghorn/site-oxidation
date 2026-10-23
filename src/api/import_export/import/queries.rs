pub const IMPORT_SITE: &str = concat!(
    "INSERT INTO sites (name, url, expected_status, expected_text, probe_interval_seconds, team_id, tls_allow_untrusted) ",
    "VALUES (?, ?, ?, ?, ?, ?, ?) ",
    "ON CONFLICT DO NOTHING"
);
