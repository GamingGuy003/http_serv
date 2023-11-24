use std::{net::{TcpListener, TcpStream}, io::Write};

use super::http_structs::{HttpMethod, HttpRequest, HttpResponse};

#[cfg(feature = "log")]
extern crate pretty_env_logger;

type HttpResponseFn = Box<dyn Fn(&HttpRequest) -> HttpResponse>;

pub struct HttpServer {
    listener: TcpListener,
    // Method Path Closure
    handlers: Vec<(HttpMethod, String, HttpResponseFn)>,
}

impl HttpServer {
    pub fn new(
        addr: String,
        port: String,
        handlers: Vec<(HttpMethod, String, HttpResponseFn)>
    ) -> Result<Self, std::io::Error> {
        Ok(Self { listener: TcpListener::bind(format!("{addr}:{port}"))?, handlers })
    }

    // main server loop that handles incomming connections
    pub fn run_loop(&self) -> std::io::Result<()> {
        // accepts connection
        for stream in self.listener.incoming() {
            let mut stream = stream?;
            #[cfg(feature = "log")]
            log::debug!("Connection from: {}", stream.peer_addr().expect("Could not resolve socket"));
            let mut http_request = HttpRequest::from_stream(&mut stream)?;
            // checks which function to run
            for handler in &self.handlers {
                let mut path_matches = true;
                let mut route_params = Vec::new();
                let split_path = http_request.http_headers.path.split_once('?');

                // split route and query params and parse query
                let (route_params_unparsed, query_params): (String, Option<Vec<(String, String)>>) = match split_path {
                    Some((rpu, qpu)) => {
                        // parse query params
                        (rpu.to_owned(), Some(http_request.query_params_from_string(qpu.to_owned())))
                    }
                    None => (http_request.http_headers.path.clone(), None),
                };

                http_request.query_params = query_params;

                let mut handler_path = handler.1.split('/').collect::<Vec<&str>>();
                let mut request_path = route_params_unparsed.split('/').collect::<Vec<&str>>();
                handler_path.retain(|x| !x.is_empty());
                request_path.retain(|x| !x.is_empty());

                // if different amount of elements, paths will never match anyways so we skip
                if handler_path.len() != request_path.len() {
                    continue;
                }

                for (handler_element, request_element) in handler_path.iter().zip(request_path.iter()) {
                    if handler_element.starts_with(':') {
                        route_params.push((handler_element.to_owned().to_owned(), request_element.to_owned().to_owned()));
                        continue;
                    }
                    if handler_element != request_element {
                        path_matches = false;
                        break;
                    }
                }

                // if there are parameters, add them to the request
                if !route_params.is_empty() {
                    http_request.route_params = Some(route_params);
                }

                if handler.0 == http_request.http_headers.method && path_matches {
                    #[cfg(feature = "log")]
                    log::info!("Handling {} for {} with {}", handler.1, stream.peer_addr().unwrap(), http_request.http_headers.path);
                    self.handle_closure(&mut stream, &http_request, &handler.2)?;
                }
            }
        }
        Ok(())
    }


    fn handle_closure(&self, stream: &mut TcpStream, request: &HttpRequest, exec: &dyn Fn(&HttpRequest) -> HttpResponse) -> std::io::Result<()> {
        let response = exec(request);
        stream.write_all(response.to_headers().join("\r\n").as_bytes())?;
        match response.data {
            Some(data) => stream.write_all(&data.data),
            None => Ok(()),
        }
    }

    pub fn get(&mut self, path: String, exec: fn(&HttpRequest) -> HttpResponse) {
        #[cfg(feature = "log")]
        log::debug!("Adding GET {path}");
        self.handlers.push((HttpMethod::GET, path, Box::from(exec)));
    }

    pub fn put(&mut self, path: String, exec: fn(&HttpRequest) -> HttpResponse) {
        #[cfg(feature = "log")]
        log::debug!("Adding PUT {path}");
        self.handlers.push((HttpMethod::PUT, path, Box::from(exec)));
    }

    pub fn post(&mut self, path: String, exec: fn(&HttpRequest) -> HttpResponse) {
        #[cfg(feature = "log")]
        log::debug!("Adding POST {path}");
        self.handlers.push((HttpMethod::POST, path, Box::from(exec)));
    }

    pub fn delete(&mut self, path: String, exec: fn(&HttpRequest) -> HttpResponse) {
        #[cfg(feature = "log")]
        log::debug!("Adding DELETE {path}");
        self.handlers.push((HttpMethod::DELETE, path, Box::from(exec)));
    }

}
