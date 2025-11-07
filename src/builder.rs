use regex::Regex;

/// Converts named placeholders (`:name`) to positional placeholders (`?`) for MySQL.
///
/// This function is used internally by `PreparedQuery` and `PreparedQueryAs`.
///
/// # Examples
///
/// ```
/// use sqlx_named_bind::builder::build_query;
///
/// let sql = build_query("SELECT * FROM users WHERE id = :id AND name = :name")?;
/// assert_eq!(sql, "SELECT * FROM users WHERE id = ? AND name = ?");
/// # Ok::<(), sqlx_named_bind::Error>(())
/// ```
pub fn build_query(template: &str) -> crate::Result<String> {
    let regex = Regex::new(r":[a-zA-Z0-9_]+")?;
    let replaced = regex.replace_all(template, "?").into_owned();
    Ok(replaced)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_query_single_param() {
        let result = build_query("SELECT * FROM users WHERE id = :id").unwrap();
        assert_eq!(result, "SELECT * FROM users WHERE id = ?");
    }

    #[test]
    fn test_build_query_multiple_params() {
        let result = build_query("SELECT * FROM users WHERE id = :id AND name = :name").unwrap();
        assert_eq!(result, "SELECT * FROM users WHERE id = ? AND name = ?");
    }

    #[test]
    fn test_build_query_repeated_params() {
        let result = build_query("SELECT * FROM users WHERE id = :id OR user_id = :id").unwrap();
        assert_eq!(result, "SELECT * FROM users WHERE id = ? OR user_id = ?");
    }

    #[test]
    fn test_build_query_no_params() {
        let result = build_query("SELECT * FROM users").unwrap();
        assert_eq!(result, "SELECT * FROM users");
    }

    #[test]
    fn test_build_query_with_underscores() {
        let result = build_query("SELECT * FROM users WHERE user_id = :user_id").unwrap();
        assert_eq!(result, "SELECT * FROM users WHERE user_id = ?");
    }
}
