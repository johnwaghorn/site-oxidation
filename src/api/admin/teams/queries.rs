pub const LIST_TEAMS: &str = concat!(
    "SELECT t.id, t.name, ",
    "COUNT(DISTINCT tm.user_id) AS member_count, ",
    "COUNT(DISTINCT s.id) AS site_count ",
    "FROM teams t ",
    "LEFT JOIN team_members tm ON t.id = tm.team_id ",
    "LEFT JOIN sites s ON t.id = s.team_id ",
    "GROUP BY t.id ",
    "ORDER BY t.id ",
    "LIMIT ? OFFSET ?"
);

pub const COUNT_TEAMS: &str = "SELECT COUNT(*) FROM teams";

pub const INSERT_TEAM: &str = "INSERT INTO teams (name) VALUES (?) RETURNING id";

pub const UPDATE_TEAM: &str = "UPDATE teams SET name = ? WHERE id = ? RETURNING id";

pub const DELETE_TEAM: &str = "DELETE FROM teams WHERE id = ? RETURNING id";

pub const COUNT_TEAM_SITES: &str = "SELECT COUNT(*) FROM sites WHERE team_id = ?";

pub const ADD_TEAM_MEMBER: &str = "INSERT INTO team_members (team_id, user_id) VALUES (?, ?)";

pub const REMOVE_TEAM_MEMBER: &str =
    "DELETE FROM team_members WHERE team_id = ? AND user_id = ? RETURNING team_id";

pub const TEAM_EXISTS: &str = "SELECT COUNT(*) FROM teams WHERE id = ?";

pub const USER_EXISTS: &str = "SELECT COUNT(*) FROM users WHERE id = ?";
