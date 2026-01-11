use std::fs::File;
use std::io::Read;
use std::io::Write;

use crate::response::{response_200, response_404, response_500};
use crate::structs::{HTTPMethod, HTTPRequest};
use crate::utils::get_directory;

pub fn file_response(request: &HTTPRequest) -> Vec<u8> {
    let directory = get_directory();
    match directory {
        Some(dir) => {
            let path = format!("{}{}", dir, request.body);
            match request.method {
                HTTPMethod::Get => get_file_response(request, path),
                HTTPMethod::Post => create_file_response(request, &path),
            }
        }
        None => response_404(&request.version),
    }
}

pub fn get_file_response(request: &HTTPRequest, path: String) -> Vec<u8> {
    let mut buf = String::new();
    let file = File::open(path);

    match file {
        Ok(mut file) => {
            let mut request = request.to_owned();
            request.content_type = Some("application/octet-stream".to_string());

            match File::read_to_string(&mut file, &mut buf) {
                Ok(_) => response_200(&request, &buf),
                Err(error) => response_500(&error.to_string()),
            }
        }
        Err(_) => response_404(&request.version),
    }
}

pub fn create_file_response(request: &HTTPRequest, path: &str) -> Vec<u8> {
    match File::create(path) {
        Ok(mut file) => match file.write_all(request.content.as_bytes()) {
            Ok(_) => format!("{} 201 Created\r\n\r\n", request.version).into_bytes(),
            Err(err) => response_500(&err.to_string()),
        },
        Err(err) => response_500(&err.to_string()),
    }
}
