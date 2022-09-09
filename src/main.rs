use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};
use std::time::Duration;
use web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(5);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

const GET:&str = "GET / HTTP/1.1";
const GET_SLEEP: &str = "GET /sleep HTTP/1.1";

fn handle_connection(mut stream: TcpStream) {
    println!("start handle");
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let file_name: &str;
    let status_line: &str;

    match request_line.as_str() {
        GET => {
            file_name = "simple_page.html";
            status_line = "HTTP/1.1 200 OK";
        },
        GET_SLEEP => {
            thread::sleep(Duration::from_secs(5));
            file_name = "simple_page.html";
            status_line = "HTTP/1.1 200 OK";
        }
        _ => {
            file_name = "error_page.html";
            status_line = "HTTP/1.1 404 NOT FOUND";
        },
    }
    let contents = fs::read_to_string(file_name).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
