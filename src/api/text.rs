pub fn blank_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

pub fn check_max_chars(value: &str, context_label: &str, max_length: usize) -> Result<(), String> {
    if value.chars().count() > max_length {
        return Err(format!(
            "{context_label} must be {max_length} characters or fewer"
        ));
    }
    Ok(())
}

pub fn optional(
    value: &str,
    context_label: &str,
    max_length: usize,
) -> Result<Option<String>, String> {
    let Some(value) = blank_to_none(value) else {
        return Ok(None);
    };
    check_max_chars(&value, context_label, max_length)?;
    Ok(Some(value))
}

pub fn required(value: &str, context_label: &str, max_length: usize) -> Result<String, String> {
    let Some(value) = blank_to_none(value) else {
        return Err(format!("{context_label} is required"));
    };
    check_max_chars(&value, context_label, max_length)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("value", Some("value"))]
    #[case(" value ", Some("value"))]
    #[case("", None)]
    #[case("   ", None)]
    fn test_blank_to_none(#[case] value: &str, #[case] expected: Option<&str>) {
        assert_eq!(blank_to_none(value).as_deref(), expected);
    }

    #[rstest]
    #[case("qwe", 3, true)]
    #[case("asdf", 3, false)]
    #[case("", 3, true)]
    fn test_check_max_chars(#[case] value: &str, #[case] max: usize, #[case] valid: bool) {
        assert_eq!(check_max_chars(value, "field", max).is_ok(), valid);
    }

    #[rstest]
    #[case(" value ", Ok(Some("value")))]
    #[case("", Ok(None))]
    #[case("   ", Ok(None))]
    #[case("qwerty", Err("field must be 5 characters or fewer"))]
    fn test_optional(#[case] value: &str, #[case] expected: Result<Option<&str>, &str>) {
        let result = optional(value, "field", 5);
        assert_eq!(
            result
                .as_ref()
                .map(Option::as_deref)
                .map_err(String::as_str),
            expected
        );
    }

    #[rstest]
    #[case(" value ", Ok("value"))]
    #[case("", Err("field is required"))]
    #[case("   ", Err("field is required"))]
    #[case("qwerty", Err("field must be 5 characters or fewer"))]
    fn test_required(#[case] value: &str, #[case] expected: Result<&str, &str>) {
        let result = required(value, "field", 5);
        assert_eq!(
            result.as_ref().map(String::as_str).map_err(String::as_str),
            expected
        );
    }
}
