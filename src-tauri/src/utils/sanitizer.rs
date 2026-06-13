// ItzamBox — Input Sanitizer
// Copyright (C) 2026 SodigTech — GPL-3.0

/// Sanitize a container name. Only allows alphanumeric, dashes, dots, underscores.
pub fn sanitize_container_name(name: &str) -> Result<String, String> {
    if name.is_empty() || name.len() > 128 {
        return Err("Container name must be 1-128 characters".into());
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err("Container name contains invalid characters".into());
    }
    Ok(name.to_string())
}

/// Sanitize an image tag (repository:tag format).
pub fn sanitize_image_tag(tag: &str) -> Result<String, String> {
    if tag.is_empty() || tag.len() > 256 {
        return Err("Image tag must be 1-256 characters".into());
    }
    if !tag.chars().all(|c| {
        c.is_alphanumeric() || c == '.' || c == '/' || c == '-' || c == '_' || c == ':' || c == '@'
    }) {
        return Err("Image tag contains invalid characters".into());
    }
    Ok(tag.to_string())
}

/// Sanitize a file path to prevent traversal attacks.
pub fn sanitize_path(path: &str) -> Result<String, String> {
    // Decode URL-encoded sequences (e.g., %2e → .)
    let decoded = decode_url_encoded(path);
    if decoded.contains("..") {
        return Err("Path traversal detected".into());
    }
    // Also check original string
    if path.contains("..") {
        return Err("Path traversal detected".into());
    }
    Ok(path.to_string())
}

/// Decode URL-encoded sequences in a path string
fn decode_url_encoded(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Container Name Tests ───

    #[test]
    fn test_valid_container_name() {
        assert!(sanitize_container_name("my-nginx-01").is_ok());
    }

    #[test]
    fn test_container_name_with_dots() {
        assert!(sanitize_container_name("com.example.app").is_ok());
    }

    #[test]
    fn test_container_name_with_underscores() {
        assert!(sanitize_container_name("my_container_prod").is_ok());
    }

    #[test]
    fn test_container_name_mixed_valid() {
        assert_eq!(
            sanitize_container_name("web-app_v2.01").unwrap(),
            "web-app_v2.01"
        );
    }

    #[test]
    fn test_container_name_with_semicolon() {
        assert!(sanitize_container_name("nginx; rm -rf /").is_err());
    }

    #[test]
    fn test_container_name_with_shell_pipe() {
        assert!(sanitize_container_name("app | cat /etc/passwd").is_err());
    }

    #[test]
    fn test_container_name_with_backtick_injection() {
        assert!(sanitize_container_name("`whoami`").is_err());
    }

    #[test]
    fn test_container_name_with_dollar_sign() {
        assert!(sanitize_container_name("$(rm -rf /)").is_err());
    }

    #[test]
    fn test_container_name_with_newline() {
        assert!(sanitize_container_name("nginx\n\rmalicious").is_err());
    }

    #[test]
    fn test_empty_container_name() {
        assert!(sanitize_container_name("").is_err());
    }

    #[test]
    fn test_container_name_too_long() {
        let long = "a".repeat(129);
        assert!(sanitize_container_name(&long).is_err());
    }

    #[test]
    fn test_container_name_exactly_max() {
        let exact = "a".repeat(128);
        assert!(sanitize_container_name(&exact).is_ok());
    }

    // ─── Image Tag Tests ───

    #[test]
    fn test_valid_image_tag() {
        assert!(sanitize_image_tag("nginx:latest").is_ok());
    }

    #[test]
    fn test_image_tag_with_registry() {
        assert!(sanitize_image_tag("registry.example.com/my-app:v1.2.3").is_ok());
    }

    #[test]
    fn test_image_tag_with_digest() {
        assert!(sanitize_image_tag("ubuntu@sha256:abc123def456").is_ok());
    }

    #[test]
    fn test_image_tag_path_style() {
        assert!(sanitize_image_tag("library/nginx:alpine").is_ok());
    }

    #[test]
    fn test_empty_image_tag() {
        assert!(sanitize_image_tag("").is_err());
    }

    #[test]
    fn test_image_tag_with_spaces() {
        assert!(sanitize_image_tag("nginx latest").is_err());
    }

    #[test]
    fn test_image_tag_with_shell_chars() {
        assert!(sanitize_image_tag("nginx;echo hacked").is_err());
    }

    #[test]
    fn test_image_tag_too_long() {
        let long = "a".repeat(257);
        assert!(sanitize_image_tag(&long).is_err());
    }

    // ─── Path Tests ───

    #[test]
    fn test_path_traversal_double_dot() {
        assert!(sanitize_path("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_path_traversal_encoded() {
        assert!(sanitize_path("/var/%2e%2e/%2e%2e/etc").is_err());
    }

    #[test]
    fn test_valid_path() {
        assert!(sanitize_path("/etc/nginx/conf.d").is_ok());
    }

    #[test]
    fn test_valid_relative_path() {
        assert!(sanitize_path("config/app.yaml").is_ok());
    }

    #[test]
    fn test_path_traversal_single_component() {
        // ".." anywhere in the path should be rejected
        assert!(sanitize_path("/var/..").is_err());
    }

    #[test]
    fn test_path_empty() {
        assert!(sanitize_path("").is_ok()); // empty path is valid
    }

    #[test]
    fn test_path_normal() {
        assert_eq!(
            sanitize_path("/home/user/project").unwrap(),
            "/home/user/project"
        );
    }
}
