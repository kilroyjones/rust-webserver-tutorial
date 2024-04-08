use std::env;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::path::PathBuf;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request_line = String::from_utf8_lossy(&buffer[..]);
    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    let safe_path = sanitize_path(path);
    let status_line = match safe_path {
        Some(_) => "HTTP/1.1 200 OK",
        None => "HTTP/1.1 400 NOT FOUND",
    };

    match safe_path {
        Some(path) => {
            if let Ok(content) = fs::read_to_string(&path) {
                let content_type = if path.ends_with(".html") {
                    "text/html"
                } else if path.ends_with(".css") {
                    "text/css"
                } else if path.ends_with(".js") {
                    "application/javascript"
                } else {
                    "text/plain"
                };
                // println!("{}-{}", content_type, content);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\n\r\n{}",
                    content_type, content
                );
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            } else {
                let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n<html><body><h1>404 Not Found</h1></body></html>";
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        }
        None => {
            let response = "HTTP/1.1 400 BAD REQUEST\r\n\r\n<html><body><h1>400 Bad Request</h1></body></html>";
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}

fn sanitize_path(path: &str) -> Option<String> {
    let base_path = env::current_dir().unwrap();
    let requested_path = base_path.join(path.trim_start_matches('/'));

    match requested_path.canonicalize() {
        Ok(resolved_path) if resolved_path.starts_with(base_path) => {
            resolved_path.to_str().map(String::from)
        }
        _ => None,
    }
}
