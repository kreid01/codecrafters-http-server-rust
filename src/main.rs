use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;
use std::net::TcpStream;

const VALID_HOST: &str = "http://localhost:4221";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = extract_url(&mut stream);
                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

#[derive(Debug)]
struct HTTPRequest {
    method: HTTPMethod,
    target: String,
    version: String,
    headers: HTTPHeaders,
}

#[derive(Debug)]
enum HTTPMethod {
    Get,
}

#[derive(Debug)]
struct HTTPHeaders {
    host: String,
    // user_agent: String,
    // accept: String,
}

fn extract_url(stream: &mut TcpStream) -> String {
    let mut buf = [0; 1024];
    let bytes_read = stream.read(&mut buf).unwrap();

    let request_str = String::from_utf8_lossy(&buf[..bytes_read]);
    let parts: Vec<&str> = request_str.split("\r\n").collect();

    let request = parts.first().unwrap();
    let headers = parts.get(1).unwrap();

    let request = get_request(request, headers);

    match request.target.as_str() {
        "/" => format!("{} 200 OK\r\n\r\n", request.version),
        _ => format!("{} 404 Not Found\r\n\r\n", request.version),
    }
}

fn get_request(request: &str, headers: &str) -> HTTPRequest {
    let request_parts: Vec<&str> = request.split_whitespace().collect();
    let headers_parts: Vec<&str> = headers.split_whitespace().collect();

    let _method = request_parts.first().unwrap();
    let target = request_parts.get(1).unwrap();
    let version = request_parts.get(2).unwrap();

    let host = headers_parts.get(1).unwrap();
    let headers = HTTPHeaders {
        host: host.to_string(),
    };

    HTTPRequest {
        method: HTTPMethod::Get,
        target: target.to_string(),
        version: version.to_string(),
        headers,
    }
}
