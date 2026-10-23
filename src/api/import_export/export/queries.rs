pub const EXPORT_SITES_ADMIN: &str = concat!(
    "SELECT name, url, expected_status, expected_text, probe_interval_seconds, tls_allow_untrusted ",
    "FROM sites ORDER BY id"
);

pub const EXPORT_SITES_USER: &str = concat!(
    "SELECT s.name, s.url, s.expected_status, s.expected_text, s.probe_interval_seconds, ",
    "s.tls_allow_untrusted ",
    "FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE tm.user_id = ? ",
    "ORDER BY s.id"
);
