use std::{net::{TcpListener, TcpStream}, io::Write};

use super::http_structs::{HttpMethod, HttpRequest, HttpResponse};

#[cfg(feature = "log")]
extern crate pretty_env_logger;

type HttpHandlerFn = Box<dyn Send + (Fn(HttpRequest) -> HttpResponse) + Sync +'static>;

/// Represents the http server
pub struct HttpServer {
    listener: TcpListener,
    #[cfg(feature = "threading")]
    threads: u32,
    // Method Path Closure
    handlers: Vec<(HttpMethod, String, HttpHandlerFn)>,
}

impl HttpServer {
    /// Creates new instance of HttpServer
    /// Examples:
    /// ```rust
    /// use http_serv::http::server::HttpServer;
    /// 
    /// let server = HttpServer::new(String::from("127.0.0.1"), String::from("8443"), Vec::new());
    /// ```
    #[cfg(not(feature = "threading"))]
    pub fn new(
        addr: String,
        port: String,
        handlers: Vec<(HttpMethod, String, HttpHandlerFn)>
    ) -> Result<Self, std::io::Error> {
        Ok(Self { listener: TcpListener::bind(format!("{addr}:{port}"))?, handlers })
    }
    /// Creates new instance of HttpServer
    /// Examples:
    /// ```rust
    /// use http_serv::http::server::HttpServer;
    /// 
    /// // If num_cpus is enabled, threads can be set as the third arg. If no number is supplied, num_cpus will assume corecount * 3
    /// #[cfg(feature = "num_cpus")]
    /// let server = HttpServer::new(String::from("127.0.0.1"), String::from("8443"), Some(10), Vec::new()).unwrap();
    /// 
    /// // If num_cpus is not enabled, thread count has to be specified
    /// #[cfg(not(feature = "num_cpus"))]
    /// let server = HttpServer::new(String::from("127.0.0.1"), String::from("8443"), 10, Vec::new()).unwrap();
    /// ```
    #[cfg(feature = "threading")]
    pub fn new(
        addr: String,
        port: String,
        #[cfg(feature = "num_cpus")]
        threads: Option<u32>,
        #[cfg(not(feature = "num_cpus"))]
        threads: u32,
        handlers: Vec<(HttpMethod, String, HttpHandlerFn)>
    ) -> Result<Self, std::io::Error> {
        #[cfg(feature = "num_cpus")]
        let threads = match threads {
            Some(threads) => threads,
            None => (num_cpus::get() as u32) * 3,
        };
        Ok(Self { listener: TcpListener::bind(format!("{addr}:{port}"))?, threads, handlers })
    }


    /// Main server loop that handles incoming connections
    /// ```ignore
    /// use http_serv::http::server::HttpServer;
    /// 
    /// let server = HttpServer::new(String::from("127.0.0.1"), String::from("8443"), Some(10), Vec::new()).unwrap();
    /// server.run_loop().unwrap();
    /// ```
    pub fn run_loop(&self) -> std::io::Result<()> {
        #[cfg(feature = "threading")]
        {
            let mut threadpool = scoped_threadpool::Pool::new(self.threads);
            threadpool.scoped(|scope| {
                // accepts connection
                for stream in self.listener.incoming() {
                    let mut stream = match stream {
                        Ok(stream) => stream,
                        Err(_err) => {
                            #[cfg(feature = "log")]
                            log::error!("Failed to get stream: {_err}");
                            continue;
                        }
                    };
                    let http_request = match HttpRequest::from_stream(&mut stream) {
                        Ok(http_request) => http_request,
                        Err(_err) => {
                            #[cfg(feature = "log")]
                            log::error!("Failed to build http_request: {_err}");
                            continue;
                        }
                    };
                    #[cfg(feature = "log")]
                    log::info!("[{}]: {}", stream.peer_addr().unwrap_or(std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 0000)), http_request.http_headers.path);
                    scope.execute(|| {
                        match handle_connection(stream, http_request, &self.handlers) {
                            Ok(_) => {},
                            Err(_err) => {
                                #[cfg(feature = "log")]
                                log::error!("Encountered error handling connection: {_err}");
                            },
                        };
                    });
                }
                
            });
        }

        #[cfg(not(feature = "threading"))]
        for stream in self.listener.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(_err) => {
                    #[cfg(feature = "log")]
                    log::error!("Failed to get stream: {_err}");
                    continue;
                }
            };
            let http_request = match HttpRequest::from_stream(&mut stream) {
                Ok(http_request) => http_request,
                Err(_err) => {
                    #[cfg(feature = "log")]
                    log::error!("Failed to build http_request: {_err}");
                    continue;
                }
            };
            #[cfg(feature = "log")]
            log::info!("[{}]: {}", stream.peer_addr().unwrap_or(std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 0000)), http_request.http_headers.path);
            match handle_connection(stream, http_request, &self.handlers) {
                Ok(_) => {},
                Err(_err) => {
                    #[cfg(feature = "log")]
                    log::error!("Encountered error handling connection: {_err}");
                },
            };
        }

        Ok(())
    }

    /// Adds a get method handler to the server
    /// Example:
    /// ```rust
    /// use http_serv::http::{server::HttpServer, http_structs::{HttpResponse, HttpData}};
    /// 
    /// let mut server = HttpServer::new("0.0.0.0".to_string(), "8443".to_string(), Vec::new()).unwrap();
    /// // :tag in a path will be used as route parameter
    /// server.get("/:uri".to_owned(), Box::new(|request| {
    ///     let mut resp = HttpResponse::default();
    ///     resp.data = Some(HttpData::new(format!("{:#?}", request).as_bytes().to_vec()));
    ///     return resp;
    /// }));
    /// ```
    pub fn get(&mut self, path: String, exec: HttpHandlerFn) {
        #[cfg(feature = "log")]
        log::debug!("Adding GET {path}");
        self.handlers.push((HttpMethod::GET, path, exec));
    }

    pub fn post(&mut self, path: String, exec: HttpHandlerFn) {
        #[cfg(feature = "log")]
        log::debug!("Adding POST {path}");
        self.handlers.push((HttpMethod::POST, path, exec));
     }
     
     pub fn put(&mut self, path: String, exec: HttpHandlerFn) {
        #[cfg(feature = "log")]
        log::debug!("Adding PUT {path}");
        self.handlers.push((HttpMethod::PUT, path, exec));
     }
     
     pub fn delete(&mut self, path: String, exec: HttpHandlerFn) {
        #[cfg(feature = "log")]
        log::debug!("Adding DELETE {path}");
        self.handlers.push((HttpMethod::DELETE, path, exec));
     }
     

    

}

fn handle_connection(mut stream: TcpStream, http_request: HttpRequest, handlers: &Vec<(HttpMethod, String, HttpHandlerFn)>) -> std::io::Result<()> {
    let mut http_request = http_request.clone();
    #[cfg(feature = "log")]
    let mut found_handler = false;
    // checks which function to run
    for handler in handlers {
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
            {
                found_handler = true;
                log::debug!("Using handler {} for {} from {}", handler.1, http_request.http_headers.path, stream.peer_addr().unwrap_or(std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 0000)));
            }
            handle_closure(&mut stream, http_request.clone(), &handler.2)?;
        }
    }
    #[cfg(feature = "log")]
    if !found_handler {
        log::warn!("Could not find handler")
    }
    Ok(())
}

fn handle_closure(stream: &mut TcpStream, request: HttpRequest, exec: &HttpHandlerFn) -> std::io::Result<()> {
    let response = exec(request);
    stream.write_all(response.to_headers().join("\r\n").as_bytes())?;
    match response.data {
        Some(data) => stream.write_all(&data.data),
        None => Ok(()),
    }
}
