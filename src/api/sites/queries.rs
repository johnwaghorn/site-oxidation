// Access checks
pub const CHECK_SITE_ACCESS_ADMIN: &str = "SELECT EXISTS(SELECT 1 FROM sites WHERE id = ?)";

pub const CHECK_SITE_ACCESS_USER: &str = concat!(
    "SELECT EXISTS(SELECT 1 FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE s.id = ? AND tm.user_id = ?)"
);

// Admin scoped
pub const SELECT_SITE_ADMIN: &str = concat!(
    "SELECT s.id, s.name, s.url, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "t.name AS team_name, s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s LEFT JOIN teams t ON t.id = s.team_id WHERE s.id = ?"
);

pub const LIST_SITES_ADMIN: &str = concat!(
    "SELECT s.id, s.name, s.url, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "t.name AS team_name, s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s LEFT JOIN teams t ON t.id = s.team_id ",
    "WHERE (?1 IS NULL OR s.name LIKE '%' || ?1 || '%' OR s.url LIKE '%' || ?1 || '%') ",
    "ORDER BY s.id DESC LIMIT ?2 OFFSET ?3"
);

pub const COUNT_SITES_ADMIN: &str = concat!(
    "SELECT COUNT(*) FROM sites s ",
    "WHERE (?1 IS NULL OR s.name LIKE '%' || ?1 || '%' OR s.url LIKE '%' || ?1 || '%')"
);

// Team scoped
pub const CHECK_TEAM_MEMBERSHIP: &str =
    "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = ? AND user_id = ?)";

pub const SELECT_SITE_USER: &str = concat!(
    "SELECT s.id, s.name, s.url, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "t.name AS team_name, s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "LEFT JOIN teams t ON t.id = s.team_id ",
    "WHERE s.id = ? AND tm.user_id = ?"
);

pub const LIST_SITES_USER: &str = concat!(
    "SELECT s.id, s.name, s.url, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "t.name AS team_name, s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "LEFT JOIN teams t ON t.id = s.team_id ",
    "WHERE tm.user_id = ?1 ",
    "AND (?2 IS NULL OR s.name LIKE '%' || ?2 || '%' OR s.url LIKE '%' || ?2 || '%') ",
    "ORDER BY s.id DESC LIMIT ?3 OFFSET ?4"
);

pub const COUNT_SITES_USER: &str = concat!(
    "SELECT COUNT(*) FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE tm.user_id = ?1 ",
    "AND (?2 IS NULL OR s.name LIKE '%' || ?2 || '%' OR s.url LIKE '%' || ?2 || '%')"
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
    "RETURNING id"
);

pub const UPDATE_SITE: &str = concat!(
    "UPDATE sites SET name=?, url=?, expected_status=?, expected_text=?, ",
    "probe_interval_seconds=?, team_id=?, tls_allow_untrusted=? ",
    "WHERE id = ? ",
    "RETURNING id"
);

pub const DELETE_SITE: &str = "DELETE FROM sites WHERE id = ?";
