pub const TEAM_EXISTS: &str = "SELECT EXISTS(SELECT 1 FROM teams WHERE id = ?)";

pub const CHECK_TEAM_MEMBERSHIP: &str =
    "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = ? AND user_id = ?)";

pub const SELECT_NOTIFICATIONS: &str = concat!(
    "SELECT t.id AS team_id, n.slack_webhook_url, n.microsoft_teams_webhook_url, ",
    "n.telegram_bot_token IS NOT NULL AS telegram_bot_token_set, n.telegram_chat_id, ",
    "n.smtp_host, n.smtp_port, COALESCE(n.smtp_security, 'starttls') AS smtp_security, ",
    "COALESCE(n.smtp_auth, 1) AS smtp_auth, n.smtp_username, n.smtp_from_email, n.smtp_to_email, ",
    "n.smtp_password IS NOT NULL AS smtp_password_set, ",
    "COALESCE(n.notify_site_down, 1) AS notify_site_down, ",
    "COALESCE(n.notify_site_recovered, 1) AS notify_site_recovered, ",
    "COALESCE(n.notify_cert_expiring, 1) AS notify_cert_expiring ",
    "FROM teams t ",
    "LEFT JOIN team_notification_settings n ON n.team_id = t.id ",
    "WHERE t.id = ?"
);

pub const UPSERT_NOTIFICATIONS: &str = concat!(
    "INSERT INTO team_notification_settings (",
    "team_id, slack_webhook_url, microsoft_teams_webhook_url, telegram_bot_token, telegram_chat_id, ",
    "smtp_host, smtp_port, smtp_security, smtp_auth, smtp_username, smtp_password, ",
    "smtp_from_email, smtp_to_email, notify_site_down, notify_site_recovered, notify_cert_expiring",
    ") VALUES (?, ?, ?, ?, ?, ?, ?, COALESCE(?, 'starttls'), COALESCE(?, 1), ?, ?, ?, ?, COALESCE(?, 1), COALESCE(?, 1), COALESCE(?, 1)) ",
    "ON CONFLICT(team_id) DO UPDATE SET ",
    "slack_webhook_url = CASE WHEN ? THEN excluded.slack_webhook_url ELSE slack_webhook_url END, ",
    "microsoft_teams_webhook_url = CASE WHEN ? THEN excluded.microsoft_teams_webhook_url ELSE microsoft_teams_webhook_url END, ",
    "telegram_bot_token = CASE WHEN ? THEN excluded.telegram_bot_token ELSE telegram_bot_token END, ",
    "telegram_chat_id = CASE WHEN ? THEN excluded.telegram_chat_id ELSE telegram_chat_id END, ",
    "smtp_host = CASE WHEN ? THEN excluded.smtp_host ELSE smtp_host END, ",
    "smtp_port = CASE WHEN ? THEN excluded.smtp_port ELSE smtp_port END, ",
    "smtp_security = CASE WHEN ? THEN excluded.smtp_security ELSE smtp_security END, ",
    "smtp_auth = CASE WHEN ? THEN excluded.smtp_auth ELSE smtp_auth END, ",
    "smtp_username = CASE WHEN ? THEN excluded.smtp_username ELSE smtp_username END, ",
    "smtp_password = CASE WHEN ? THEN excluded.smtp_password ELSE smtp_password END, ",
    "smtp_from_email = CASE WHEN ? THEN excluded.smtp_from_email ELSE smtp_from_email END, ",
    "smtp_to_email = CASE WHEN ? THEN excluded.smtp_to_email ELSE smtp_to_email END, ",
    "notify_site_down = CASE WHEN ? THEN excluded.notify_site_down ELSE notify_site_down END, ",
    "notify_site_recovered = CASE WHEN ? THEN excluded.notify_site_recovered ELSE notify_site_recovered END, ",
    "notify_cert_expiring = CASE WHEN ? THEN excluded.notify_cert_expiring ELSE notify_cert_expiring END, ",
    "updated_at = CURRENT_TIMESTAMP ",
    "RETURNING team_id, slack_webhook_url, microsoft_teams_webhook_url, ",
    "telegram_bot_token IS NOT NULL AS telegram_bot_token_set, telegram_chat_id, ",
    "smtp_host, smtp_port, smtp_security, smtp_auth, smtp_username, smtp_from_email, smtp_to_email, ",
    "smtp_password IS NOT NULL AS smtp_password_set, ",
    "notify_site_down, notify_site_recovered, notify_cert_expiring"
);
