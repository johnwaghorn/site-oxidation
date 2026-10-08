pub const UPDATE_THEME_PREFERENCE: &str = "UPDATE users SET theme_preference = ? WHERE id = ?";

pub const SELECT_USER_TEAMS: &str = concat!(
    "SELECT t.id, t.name ",
    "FROM teams t ",
    "JOIN team_members tm ON t.id = tm.team_id ",
    "WHERE tm.user_id = ? ",
    "ORDER BY t.name"
);

pub const SELECT_ALL_TEAMS: &str = "SELECT id, name FROM teams ORDER BY name";
