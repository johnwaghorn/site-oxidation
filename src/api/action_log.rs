macro_rules! log_action_by_user {
    ($user:expr, $message:literal $(, $($what_was_affected:tt)+)?) => {
        tracing::info!(
            action_by = %$user.username,
            action_by_user_id = $user.id,
            $($($what_was_affected)+,)?
            $message
        )
    };
}

macro_rules! log_refused_action_by_user {
    ($user:expr, $message:literal $(, $($what_was_refused:tt)+)?) => {
        tracing::warn!(
            action_by = %$user.username,
            action_by_user_id = $user.id,
            $($($what_was_refused)+,)?
            $message
        )
    };
}

pub(crate) use log_action_by_user;
pub(crate) use log_refused_action_by_user;

pub fn id_or_none(id: Option<i64>) -> String {
    id.map_or_else(|| "none".to_owned(), |id| id.to_string())
}
