use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, thread};

use hello::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.run(|| handle_connection(stream));
    }

    // TODO: join all threads in the pool...?
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    // TODO: proper buffer handling + read_all ??
    stream.read(&mut buffer).unwrap();

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let root_get = b"GET / HTTP/1.1\r\n";
    let sleep_get = b"GET /sleep HTTP/1.1\r\n";
    // let favicon_get = b"GET /favicon.ico HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(root_get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep_get) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let content = fs::read_to_string(filename).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );
    // TODO: proper buffer handling + write_all ??
    stream.write(response.as_bytes()).unwrap();
}
