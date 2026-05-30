// Access checks
pub const CHECK_SITE_ACCESS_ADMIN: &str = "SELECT EXISTS(SELECT 1 FROM sites WHERE id = ?)";

pub const CHECK_SITE_ACCESS_USER: &str = concat!(
    "SELECT EXISTS(SELECT 1 FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE s.id = ? AND tm.user_id = ?)"
);

// Admin scoped
pub const SELECT_SITE_ADMIN: &str = concat!(
    "SELECT id, name, url, expected_status, expected_text, status, ",
    "last_checked_at, last_response_time_ms, probe_interval_seconds, team_id, ",
    "tls_allow_untrusted, cert_status, cert_expires_at ",
    "FROM sites WHERE id = ?"
);

pub const LIST_SITES_ADMIN: &str = concat!(
    "SELECT id, name, url, expected_status, expected_text, status, ",
    "last_checked_at, last_response_time_ms, probe_interval_seconds, team_id, ",
    "tls_allow_untrusted, cert_status, cert_expires_at ",
    "FROM sites ORDER BY id DESC LIMIT ? OFFSET ?"
);

pub const COUNT_SITES_ADMIN: &str = "SELECT COUNT(*) FROM sites";

// Team scoped
pub const CHECK_TEAM_MEMBERSHIP: &str =
    "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = ? AND user_id = ?)";

pub const SELECT_SITE_USER: &str = concat!(
    "SELECT s.id, s.name, s.url, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE s.id = ? AND tm.user_id = ?"
);

pub const LIST_SITES_USER: &str = concat!(
    "SELECT s.id, s.name, s.url, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE tm.user_id = ? ",
    "ORDER BY s.id DESC LIMIT ? OFFSET ?"
);

pub const COUNT_SITES_USER: &str = concat!(
    "SELECT COUNT(*) FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE tm.user_id = ?"
);

// Outages
pub const LIST_OUTAGES: &str = concat!(
    "SELECT id, site_id, http_status, started_at, ended_at, error_message ",
    "FROM outages WHERE site_id = ? ",
    "ORDER BY started_at DESC, id DESC LIMIT ? OFFSET ?"
);

pub const COUNT_OUTAGES: &str = "SELECT COUNT(*) FROM outages WHERE site_id = ?";

// Mutations
pub const INSERT_SITE: &str = concat!(
    "INSERT INTO sites (name, url, expected_status, expected_text, probe_interval_seconds, team_id, tls_allow_untrusted) ",
    "VALUES (?, ?, ?, ?, ?, ?, ?) ",
    "RETURNING id, name, url, expected_status, expected_text, status, ",
    "last_checked_at, last_response_time_ms, probe_interval_seconds, team_id, ",
    "tls_allow_untrusted, cert_status, cert_expires_at"
);

pub const UPDATE_SITE: &str = concat!(
    "UPDATE sites SET name=?, url=?, expected_status=?, expected_text=?, ",
    "probe_interval_seconds=?, team_id=?, tls_allow_untrusted=? ",
    "WHERE id = ? ",
    "RETURNING id, name, url, expected_status, expected_text, status, ",
    "last_checked_at, last_response_time_ms, probe_interval_seconds, team_id, ",
    "tls_allow_untrusted, cert_status, cert_expires_at"
);

pub const DELETE_SITE: &str = "DELETE FROM sites WHERE id = ?";
