use std::{net::{TcpListener, TcpStream}, io::Write};

use super::http_structs::{HttpMethod, HttpRequest, HttpResponse};

pub struct HttpServer {
    listener: TcpListener,
    // Method Path Closure
    handlers: Vec<(HttpMethod, String, Box<dyn Fn(&HttpRequest) -> HttpResponse>)>,
}

impl HttpServer {
    pub fn new(
        addr: String,
        port: String,
        handlers: Vec<(HttpMethod, String, Box<dyn Fn(&HttpRequest) -> HttpResponse>
    )>) -> Result<Self, std::io::Error> {
        Ok(Self { listener: TcpListener::bind(format!("{addr}:{port}"))?, handlers })
    }

    // main server loop that handles incomming connections
    pub fn run_loop(&self) -> std::io::Result<()> {
        // accepts connection
        for stream in self.listener.incoming() {
            let mut stream = stream?;
            let http_request = &HttpRequest::from_stream(&mut stream)?;
            // checks which function to run
            for handler in &self.handlers {
                if handler.0 == http_request.http_headers.method && handler.1 == http_request.http_headers.path {
                    self.handle_closure(&mut stream, http_request, &handler.2)?;
                }
            }
        }
        Ok(())
    }


    fn handle_closure(&self, stream: &mut TcpStream, mut request: &HttpRequest, exec: &Box<dyn Fn(&HttpRequest) -> HttpResponse>) -> std::io::Result<()> {
        let response = exec(request);
        stream.write_all(response.to_headers().join("\r\n").as_bytes())?;
        match response.data {
            Some(data) => stream.write_all(&data.data),
            None => Ok(()),
        }
    }

    pub fn get(&mut self, path: String, exec: fn(&HttpRequest) -> HttpResponse) {
        self.handlers.push((HttpMethod::GET, path, Box::from(exec)));
    }

    pub fn put(&mut self, path: String, exec: fn(&HttpRequest) -> HttpResponse) {
        self.handlers.push((HttpMethod::PUT, path, Box::from(exec)));
    }

    pub fn post(&mut self, path: String, exec: fn(&HttpRequest) -> HttpResponse) {
        self.handlers.push((HttpMethod::POST, path, Box::from(exec)));
    }

    pub fn delete(&mut self, path: String, exec: fn(&HttpRequest) -> HttpResponse) {
        self.handlers.push((HttpMethod::DELETE, path, Box::from(exec)));
    }

}