use std::env;

use crate::structs::HTTPMethod;

pub fn get_request_property(parts: &[&str], property: &str) -> Option<String> {
    for part in parts {
        if part.contains(property) {
            let replace = format!("{}: ", property);
            return Some(part.replace(&replace, ""));
        }
    }

    None
}

pub fn get_http_method(method: &str) -> HTTPMethod {
    match method.to_lowercase().as_str() {
        "post" => HTTPMethod::Post,
        _ => HTTPMethod::Get,
    }
}

pub fn get_directory() -> Option<String> {
    let variables: Vec<String> = env::args().collect();

    if let Some(index) = variables.iter().position(|v| v == "--directory") {
        match variables.get(index + 1) {
            Some(dir) => return Some(dir.to_string()),
            None => return None,
        }
    };

    None
}

pub fn get_connection_header(header: &Option<String>) -> String {
    match header {
        Some(status) => format!("\r\nConnection: {}", status),
        None => "".to_string(),
    }
}
