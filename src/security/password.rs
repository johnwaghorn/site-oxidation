use crate::api::errors::ApiErrorResponse;

const MIN_PASSWORD_LENGTH: usize = 12;
const MAX_PASSWORD_LENGTH: usize = 128;

pub fn validate_password_bounds(password: &str) -> Result<(), ApiErrorResponse> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(ApiErrorResponse::validation(&format!(
            "Password must be at least {MIN_PASSWORD_LENGTH} characters"
        )));
    }
    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ApiErrorResponse::validation(&format!(
            "Password must be at most {MAX_PASSWORD_LENGTH} characters"
        )));
    }
    Ok(())
}

pub fn validate_password_not_username(
    password: &str,
    username: &str,
) -> Result<(), ApiErrorResponse> {
    if password.eq_ignore_ascii_case(username) {
        return Err(ApiErrorResponse::validation(
            "Password must not be the same as your username",
        ));
    }
    Ok(())
}

pub fn validate_password_changed(
    new_password: &str,
    current_password: &str,
) -> Result<(), ApiErrorResponse> {
    if new_password == current_password {
        return Err(ApiErrorResponse::validation(
            "New password must be different from current password",
        ));
    }
    Ok(())
}
