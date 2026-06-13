// ItzamBox — Input Sanitizer
// Copyright (C) 2026 SodigTech — GPL-3.0

/// Sanitize a container name. Only allows alphanumeric, dashes, dots, underscores.
pub fn sanitize_container_name(name: &str) -> Result<String, String> {
    if name.is_empty() || name.len() > 128 {
        return Err("Container name must be 1-128 characters".into());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err("Container name contains invalid characters".into());
    }
    Ok(name.to_string())
}

/// Sanitize an image tag (repository:tag format).
pub fn sanitize_image_tag(tag: &str) -> Result<String, String> {
    if tag.is_empty() || tag.len() > 256 {
        return Err("Image tag must be 1-256 characters".into());
    }
    if !tag.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '/' || c == '-' || c == '_' || c == ':') {
        return Err("Image tag contains invalid characters".into());
    }
    Ok(tag.to_string())
}

/// Sanitize a file path to prevent traversal attacks.
pub fn sanitize_path(path: &str) -> Result<String, String> {
    if path.contains("..") {
        return Err("Path traversal detected".into());
    }
    Ok(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_container_name() {
        assert!(sanitize_container_name("my-nginx-01").is_ok());
    }

    #[test]
    fn test_container_name_with_semicolon() {
        assert!(sanitize_container_name("nginx; rm -rf /").is_err());
    }

    #[test]
    fn test_empty_container_name() {
        assert!(sanitize_container_name("").is_err());
    }

    #[test]
    fn test_valid_image_tag() {
        assert!(sanitize_image_tag("nginx:latest").is_ok());
    }

    #[test]
    fn test_path_traversal() {
        assert!(sanitize_path("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_valid_path() {
        assert!(sanitize_path("/etc/nginx/conf.d").is_ok());
    }
}
