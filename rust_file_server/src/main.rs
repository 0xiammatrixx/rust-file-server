use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;
use infer;
use url_escape::decode;

fn main() {
    // Start the TCP server
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server is running on http://127.0.0.1:7878");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_client(stream);
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Convert request to String
    let request = String::from_utf8_lossy(&buffer[..]);

    // Parse the requested path
    let requested_path = parse_request(&request);

    // Prevent backtracking
    if prevent_backtracking(&requested_path) {
        // Generate HTML response based on requested path
        let response = generate_html_response(requested_path);
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {
        let response = "HTTP/1.1 403 FORBIDDEN\r\n\r\nForbidden!";
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

fn parse_request(request: &str) -> PathBuf {
    let lines: Vec<&str> = request.split("\r\n").collect();
    let parts: Vec<&str> = lines[0].split_whitespace().collect();
    let path = parts[1];
    let decoded_path = url_escape::decode(path).into_owned();

    PathBuf::from(decoded_path)
}

fn prevent_backtracking(requested_path: &PathBuf) -> bool {
    let rootcwd = env::current_dir().unwrap();
    let rootcwd_len = rootcwd.canonicalize().unwrap().components().count();
    let resource = rootcwd.join(requested_path);
    let resource_len = resource.canonicalize().unwrap().components().count();

    rootcwd_len <= resource_len
}

fn generate_html_response(requested_path: PathBuf) -> String {
    if requested_path.is_dir() {
        let file_list = list_files(requested_path);
        let html = generate_html(file_list, "Directory");
        format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", html)
    } else if requested_path.is_file() {
        let content = fs::read_to_string(requested_path).unwrap_or_else(|_| "Unable to read file.".to_string());
        format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}", content)
    } else {
        "HTTP/1.1 404 NOT FOUND\r\n\r\nFile not found!".to_string()
    }
}

fn list_files(dir: PathBuf) -> String {
    let mut file_list = String::new();
    for entry in WalkDir::new(dir).min_depth(1).max_depth(1) {
        let entry = entry.unwrap();
        let path = entry.path();
        let display = path.display();
        file_list.push_str(&format!("<a href=\"{}\">{}</a><br>", display, display));
    }
    file_list
}

fn generate_html(file_list: String, current_dir: &str) -> String {
    let begin_html = r#"
        <!DOCTYPE html>
        <html>
        <head><meta charset="utf-8"></head>
        <body>"#.to_string();
    let header = format!("<h1>Currently in {}</h1>", current_dir);
    let end_html = r#"
        </body>
        </html>"#.to_string();
    
    format!("{}{}{}{}", begin_html, header, file_list, end_html)
}
