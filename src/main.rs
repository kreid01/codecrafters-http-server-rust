use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::{env, thread};
mod structs;

use crate::structs::{HTTPHeaders, HTTPMethod, HTTPRequest};

// remove unwrap -> proper handling * options
// get rid of clones

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let stream = Arc::new(Mutex::new(stream));
                thread::spawn(move || {
                    let response = extract_url(&mut stream.lock().unwrap());
                    println!("{}", response);
                    stream
                        .lock()
                        .unwrap()
                        .write_all(response.as_bytes())
                        .unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn extract_url(stream: &mut TcpStream) -> String {
    let mut buf = [0; 1024];
    let bytes_read = stream.read(&mut buf).unwrap();

    let request_str = String::from_utf8_lossy(&buf[..bytes_read]);
    let parts: Vec<&str> = request_str.split("\r\n").collect();

    let request = get_request(parts);

    match request.target.as_str() {
        "/" => format!("{} 200 OK\r\n\r\n", request.version),
        "/echo" => response_200(request.clone(), &request.body.clone()),
        "/user-agent" => response_200(request.clone(), &request.headers.user_agent.clone()),
        "/files" => file_response(request.clone()),
        _ => not_found(request.version),
    }
}

fn not_found(version: String) -> String {
    format!("{} 404 Not Found\r\n\r\n", version)
}

fn file_response(request: HTTPRequest) -> String {
    let directory = get_directory();
    match directory {
        Some(dir) => {
            let path = format!("{}{}", dir, request.body);
            match request.method {
                HTTPMethod::Get => get_file_response(request, path),
                HTTPMethod::Post => create_file_response(request, path),
            }
        }
        None => not_found(request.version),
    }
}

fn get_file_response(request: HTTPRequest, path: String) -> String {
    let mut buf = String::new();
    let file = File::open(path);

    match file {
        Ok(mut file) => {
            let mut request = request;
            request.content_type = "application/octet-stream".to_string();

            File::read_to_string(&mut file, &mut buf).unwrap();
            response_200(request, &buf)
        }
        Err(_) => not_found(request.version),
    }
}

fn create_file_response(request: HTTPRequest, path: String) -> String {
    let mut file = File::create(path).unwrap();
    file.write_all(&request.content.into_bytes()).unwrap();
    format!("{} 201 Created\r\n\r\n", request.version)
}

fn response_200(request: HTTPRequest, body: &String) -> String {
    let encoding = match request.encoding {
        Some(encoding) if encoding == "gzip" => format!("Content-Encoding: {}\r\n", encoding),
        _ => "".to_string(),
    };

    format!(
        "{} 200 OK\r\nContent-Type: {}\r\n{}Content-Length: {}\r\n\r\n{}",
        request.version,
        request.content_type,
        encoding,
        body.len(),
        body
    )
}

fn get_request(parts: Vec<&str>) -> HTTPRequest {
    let request = parts.first().unwrap();
    let headers = parts.get(1).unwrap();
    let agent = parts.get(2).unwrap();
    let content = parts.get(5).unwrap_or(&"").to_string();

    let request_parts: Vec<&str> = request.split_whitespace().collect();
    let headers_parts: Vec<&str> = headers.split_whitespace().collect();
    let agent_parts: Vec<&str> = agent.split_whitespace().collect();

    let method = request_parts.first().unwrap();
    let method = get_http_method(method);

    let target = request_parts.get(1).unwrap();
    let target_parts: Vec<&str> = target.split('/').collect();

    let target = format!("/{}", target_parts.get(1).unwrap_or(&""));
    let body = target_parts.get(2).unwrap_or(&"").to_string();

    let encoding = get_request_property(parts, "Accept-Encoding");

    let version = request_parts.get(2).unwrap();

    let host = headers_parts.get(1).unwrap();
    let user_agent = agent_parts.get(1).unwrap_or(&"").to_string();

    let headers = HTTPHeaders {
        host: host.to_string(),
        user_agent,
    };

    HTTPRequest {
        method,
        target,
        content_type: "text/plain".to_string(),
        version: version.to_string(),
        headers,
        body,
        content,
        encoding,
    }
}

fn get_request_property(parts: Vec<&str>, property: &str) -> Option<String> {
    for part in parts {
        if part.contains(property) {
            let replace = format!("{}: ", property);
            return Some(part.replace(&replace, ""));
        }
    }

    None
}

fn get_http_method(method: &str) -> HTTPMethod {
    match method.to_lowercase().as_str() {
        "post" => HTTPMethod::Post,
        _ => HTTPMethod::Get,
    }
}

fn get_directory() -> Option<String> {
    let variables: Vec<String> = env::args().collect();

    if let Some(index) = variables.iter().position(|v| v == "--directory") {
        return Some(variables.get(index + 1).unwrap().to_string());
    }

    None
}
