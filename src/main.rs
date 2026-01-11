#![deny(clippy::unwrap_used)]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

mod files;
mod response;
mod structs;
mod utils;

use crate::files::file_response;
use crate::response::{response_200, response_404};
use crate::structs::{HTTPHeaders, HTTPRequest};
use crate::utils::{get_connection_header, get_http_method, get_request_property};

// errors return 500 result

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:4221").expect("Unable to bind to port 127.0.0.1:4221");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    loop {
                        let response = extract_url(&mut stream);
                        let res = stream.write_all(&response);

                        if res.is_err() {
                            break;
                        }

                        if is_connection_closed(response) {
                            break;
                        }
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn is_connection_closed(response: Vec<u8>) -> bool {
    let response_string = String::from_utf8(response);
    match response_string {
        Ok(response_string) => response_string.contains("Connection: close"),
        Err(_) => true,
    }
}

fn extract_url(stream: &mut TcpStream) -> Vec<u8> {
    let mut buf = [0; 1024];
    let bytes_read = stream.read(&mut buf).expect("Failed to read stream buffer");

    let request_str = String::from_utf8_lossy(&buf[..bytes_read]);
    let parts: Vec<&str> = request_str.split("\r\n").collect();

    let request = get_request(parts);

    let connection = get_connection_header(&request.connection.clone());
    match request.target.as_str() {
        "/" => format!("{} 200 OK{}\r\n\r\n", request.version, connection).into_bytes(),
        "/echo" => response_200(&request, &request.body),
        "/user-agent" => response_200(&request, &request.headers.user_agent),
        "/files" => file_response(&request),
        _ => response_404(&request.version),
    }
}

fn get_request(parts: Vec<&str>) -> HTTPRequest {
    let request = parts.first().unwrap_or(&"");
    let headers = parts.get(1).unwrap_or(&"");
    let content = parts.get(5).unwrap_or(&"").to_string();

    let request_parts: Vec<&str> = request.split_whitespace().collect();
    let headers_parts: Vec<&str> = headers.split_whitespace().collect();

    let method = request_parts.first().unwrap_or(&"");
    let method = get_http_method(method);

    let target = request_parts.get(1).unwrap_or(&"");
    let target_parts: Vec<&str> = target.split('/').collect();

    let target = format!("/{}", target_parts.get(1).unwrap_or(&""));
    let body = target_parts.get(2).unwrap_or(&"").to_string();

    let encoding = get_request_property(&parts, "Accept-Encoding");
    let connection = get_request_property(&parts, "Connection");
    let content_type = get_request_property(&parts, "Content-Type");
    let user_agent = get_request_property(&parts, "User-Agent");

    let version = request_parts.get(2).unwrap_or(&"");
    let host = headers_parts.get(1).unwrap_or(&"");

    let headers = HTTPHeaders {
        _host: host.to_string(),
        user_agent: user_agent.unwrap_or("".to_string()),
    };

    HTTPRequest {
        method,
        target,
        content_type,
        version: version.to_string(),
        connection,
        headers,
        body,
        content,
        encoding,
    }
}
