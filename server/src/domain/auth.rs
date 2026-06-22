pub fn parse_bearer_token(header_value: &str) -> Option<&str> {
    let token = header_value.strip_prefix("Bearer ")?;
    if token.is_empty() { None } else { Some(token) }
}
