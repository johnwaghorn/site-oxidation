// Access checks
pub const CHECK_SITE_ACCESS_ADMIN: &str = "SELECT EXISTS(SELECT 1 FROM sites WHERE id = ?)";

pub const CHECK_SITE_ACCESS_USER: &str = concat!(
    "SELECT EXISTS(SELECT 1 FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE s.id = ? AND tm.user_id = ?)"
);

// Admin scoped
pub const SELECT_SITE_ADMIN: &str = concat!(
    "SELECT s.id, s.name, s.url, s.created_at, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "t.name AS team_name, s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s LEFT JOIN teams t ON t.id = s.team_id WHERE s.id = ?"
);

pub const LIST_SITES_ADMIN: &str = concat!(
    "SELECT s.id, s.name, s.url, s.created_at, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "t.name AS team_name, s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s LEFT JOIN teams t ON t.id = s.team_id ",
    "WHERE (?1 IS NULL OR s.name LIKE '%' || ?1 || '%' OR s.url LIKE '%' || ?1 || '%') ",
    "AND (?2 IS NULL OR (?2 = 'up' AND s.status = 'up') ",
    "  OR (?2 = 'down' AND s.status IN ('down', 'blocked'))) ",
    "ORDER BY s.id DESC LIMIT ?3 OFFSET ?4"
);

pub const COUNT_SITES_ADMIN: &str = concat!(
    "SELECT COUNT(*) FROM sites s ",
    "WHERE (?1 IS NULL OR s.name LIKE '%' || ?1 || '%' OR s.url LIKE '%' || ?1 || '%') ",
    "AND (?2 IS NULL OR (?2 = 'up' AND s.status = 'up') ",
    "  OR (?2 = 'down' AND s.status IN ('down', 'blocked')))"
);

pub const COUNT_UNASSIGNED_SITES: &str = "SELECT COUNT(*) FROM sites WHERE team_id IS NULL";

pub const SELECT_SITE_USER: &str = concat!(
    "SELECT s.id, s.name, s.url, s.created_at, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "t.name AS team_name, s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "LEFT JOIN teams t ON t.id = s.team_id ",
    "WHERE s.id = ? AND tm.user_id = ?"
);

pub const LIST_SITES_USER: &str = concat!(
    "SELECT s.id, s.name, s.url, s.created_at, s.expected_status, s.expected_text, s.status, ",
    "s.last_checked_at, s.last_response_time_ms, s.probe_interval_seconds, s.team_id, ",
    "t.name AS team_name, s.tls_allow_untrusted, s.cert_status, s.cert_expires_at ",
    "FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "LEFT JOIN teams t ON t.id = s.team_id ",
    "WHERE tm.user_id = ?1 ",
    "AND (?2 IS NULL OR s.name LIKE '%' || ?2 || '%' OR s.url LIKE '%' || ?2 || '%') ",
    "AND (?3 IS NULL OR (?3 = 'up' AND s.status = 'up') ",
    "  OR (?3 = 'down' AND s.status IN ('down', 'blocked'))) ",
    "ORDER BY s.id DESC LIMIT ?4 OFFSET ?5"
);

pub const COUNT_SITES_USER: &str = concat!(
    "SELECT COUNT(*) FROM sites s ",
    "INNER JOIN team_members tm ON s.team_id = tm.team_id ",
    "WHERE tm.user_id = ?1 ",
    "AND (?2 IS NULL OR s.name LIKE '%' || ?2 || '%' OR s.url LIKE '%' || ?2 || '%') ",
    "AND (?3 IS NULL OR (?3 = 'up' AND s.status = 'up') ",
    "  OR (?3 = 'down' AND s.status IN ('down', 'blocked')))"
);

// Outages
pub const LIST_OUTAGES: &str = concat!(
    "SELECT id, site_id, http_status, expected_status, started_at, ended_at, error_message ",
    "FROM outages WHERE site_id = ? ",
    "ORDER BY started_at DESC, id DESC LIMIT ? OFFSET ?"
);

pub const COUNT_OUTAGES: &str = "SELECT COUNT(*) FROM outages WHERE site_id = ?";

// Uptime
pub const LIST_OUTAGES_IN_UPTIME_CALCULATION_WINDOW_FOR_SITE_IDS: &str = concat!(
    "SELECT site_id, started_at, ended_at FROM outages ",
    "WHERE site_id IN (SELECT value FROM json_each(?)) AND (ended_at IS NULL OR ended_at > ?)"
);

pub const SELECT_SITE_URL_CREATED_AT_AND_LAST_CHECKED_AT: &str =
    "SELECT url, created_at, last_checked_at FROM sites WHERE id = ?";

pub const LIST_SITE_OUTAGES_IN_UPTIME_CALCULATION_WINDOW: &str = concat!(
    "SELECT started_at, ended_at FROM outages ",
    "WHERE site_id = ? AND (ended_at IS NULL OR ended_at > ?) ",
    "ORDER BY started_at"
);

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
