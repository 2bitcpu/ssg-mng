use std::borrow::Cow;
use validator::ValidationError;

pub(crate) fn validate_account(account: &str) -> Result<(), ValidationError> {
    if account.len() < 6 || account.len() > 32 {
        let mut err = ValidationError::new("account_length");
        err.message = Some(Cow::Borrowed(
            "Account ID must be between 6 and 32 characters",
        ));
        return Err(err);
    }

    let allowed = regex::Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap();
    if !allowed.is_match(account) {
        let mut err = ValidationError::new("account_charset");
        err.message = Some(Cow::Borrowed(
            "Account ID can only contain letters, numbers, ., _, -",
        ));
        return Err(err);
    }

    let consecutive_symbols = regex::Regex::new(r"[._-]{3,}").unwrap();
    if consecutive_symbols.is_match(account) {
        let mut err = ValidationError::new("account_consecutive_symbols");
        err.message = Some(Cow::Borrowed(
            "Symbols cannot appear more than twice consecutively",
        ));
        return Err(err);
    }

    Ok(())
}

pub(crate) fn validate_password(password: &str) -> Result<(), ValidationError> {
    // 長さチェック
    if password.len() < 8 || password.len() > 64 {
        let mut err = ValidationError::new("length");
        err.message = Some("Password must be 8-64 characters".into());
        return Err(err);
    }

    // ASCII文字のみチェック
    if !password.is_ascii() {
        let mut err = ValidationError::new("ascii_only");
        err.message = Some("Password must contain only ASCII characters".into());
        return Err(err);
    }

    // 文字種チェック
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password
        .chars()
        .any(|c| "!@#$%^&*()-_=+[]{}|;:,.<>?/".contains(c));

    if !(has_upper && has_lower && has_digit && has_symbol) {
        let mut err = ValidationError::new("complexity");
        err.message = Some(
            "Password must contain at least one uppercase, one lowercase, one number, and one symbol"
                .into(),
        );
        return Err(err);
    }

    Ok(())
}
