use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

use crate::structs::HTTPRequest;
use crate::utils::get_connection_header;

pub fn response_404(version: &String) -> Vec<u8> {
    format!("{} 404 Not Found\r\n\r\n", version).into_bytes()
}

pub fn response_500(error: &String) -> Vec<u8> {
    format!("500 Internal Server Error{}\r\n", error).into_bytes()
}

pub fn response_200(request: &HTTPRequest, body: &str) -> Vec<u8> {
    let body = body.trim_end_matches(|c| c == '\n' || c == '\r');

    let use_gzip = matches!(request.encoding.as_deref(), Some(enc) if enc.contains("gzip") && !enc.contains("invalid"));

    let body_bytes = if use_gzip {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        if let Err(e) = encoder.write_all(body.as_bytes()) {
            return response_500(&format!("Failed to gzip body: {}", e));
        }
        match encoder.finish() {
            Ok(b) => b,
            Err(e) => return response_500(&format!("Failed to finish gzip: {}", e)),
        }
    } else {
        body.as_bytes().to_vec()
    };

    let encoding_header = if use_gzip {
        "Content-Encoding: gzip\r\n"
    } else {
        ""
    };

    let connection_header = get_connection_header(&request.connection);
    let content_type = request.content_type.as_deref().unwrap_or("text/plain");

    let headers = format!(
        "{} 200 OK\r\nContent-Type: {}\r\n{}Content-Length: {}{}\r\n\r\n",
        request.version,
        content_type,
        encoding_header,
        body_bytes.len(),
        connection_header
    );

    let mut response = headers.into_bytes();
    response.extend_from_slice(&body_bytes);
    response
}
