pub const MIN_PASSWORD_LEN: usize = 8;
pub const MAX_DISPLAY_NAME_LEN: usize = 100;

pub fn normalize_email(raw: &str) -> String {
    raw.to_lowercase()
}

pub fn validate_signup(email: &str, password: &str, display_name: &str) -> Result<(), String> {
    if !is_plausible_email(email) {
        return Err("email format is invalid".to_string());
    }
    if password.len() < MIN_PASSWORD_LEN {
        return Err(format!("password must be at least {MIN_PASSWORD_LEN} characters"));
    }
    let name = display_name.trim();
    if name.is_empty() {
        return Err("display_name must not be empty".to_string());
    }
    if name.chars().count() > MAX_DISPLAY_NAME_LEN {
        return Err(format!("display_name must be at most {MAX_DISPLAY_NAME_LEN} characters"));
    }
    Ok(())
}

fn is_plausible_email(email: &str) -> bool {
    let email = email.trim();
    let mut parts = email.split('@');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(local), Some(domain), None)
            if !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
    )
}
