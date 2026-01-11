use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

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

    println!("{:?}", request);

    match request.target.as_str() {
        "/" => format!("{} 200 OK\r\n\r\n", request.version),
        "/echo" => response_200(request.clone(), &request.body.clone()),
        "/user-agent" => response_200(request.clone(), &request.headers.user_agent.clone()),
        _ => format!("{} 404 Not Found\r\n\r\n", request.version),
    }
}

fn response_200(request: HTTPRequest, body: &String) -> String {
    format!(
        "{} 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        request.version,
        body.len(),
        body
    )
}

fn get_request(parts: Vec<&str>) -> HTTPRequest {
    println!("{:?}", parts);

    let request = parts.first().unwrap();
    let headers = parts.get(1).unwrap();
    let agent = parts.get(2).unwrap();

    let request_parts: Vec<&str> = request.split_whitespace().collect();
    let headers_parts: Vec<&str> = headers.split_whitespace().collect();
    let agent_parts: Vec<&str> = agent.split_whitespace().collect();

    let _method = request_parts.first().unwrap();

    let target = request_parts.get(1).unwrap();
    let target_parts: Vec<&str> = target.split('/').collect();

    let target = format!("/{}", target_parts.get(1).unwrap_or(&""));
    let body = target_parts.get(2).unwrap_or(&"").to_string();

    let version = request_parts.get(2).unwrap();

    println!("{:?}", headers_parts);

    let host = headers_parts.get(1).unwrap();
    let user_agent = agent_parts.get(1).unwrap_or(&"").to_string();

    let headers = HTTPHeaders {
        host: host.to_string(),
        user_agent,
        accept: "".to_string(),
    };

    HTTPRequest {
        method: HTTPMethod::Get,
        target,
        version: version.to_string(),
        headers,
        body,
    }
}

#[derive(Debug, Clone)]
struct HTTPRequest {
    method: HTTPMethod,
    target: String,
    version: String,
    headers: HTTPHeaders,
    body: String,
}

#[derive(Debug, Clone)]
enum HTTPMethod {
    Get,
}

#[derive(Debug, Clone)]
struct HTTPHeaders {
    host: String,
    user_agent: String,
    accept: String,
}
