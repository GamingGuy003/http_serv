use std::{
    io::{self, Write},
    net::{TcpListener, TcpStream},
};

use http_base::http::http_structs::{HttpMethod, HttpRequest, HttpResponse};

#[cfg(feature = "log")]
extern crate pretty_env_logger;

type HttpHandlerFn = Box<dyn (Fn(&HttpRequest) -> HttpResponse) + Sync + Send + 'static>;

/// Represents the http server
pub struct HttpServer {
    listener: TcpListener,
    #[cfg(feature = "threading")]
    threads: u32,
    // Method Path Closure
    handlers: Vec<(HttpMethod, String, HttpHandlerFn)>,
    default_handler: HttpHandlerFn,
}

impl HttpServer {
    /// Creates new instance of HttpServer
    /// Examples:
    /// ```rust
    /// use http_serv::http_server::server::HttpServer;
    /// 
    /// let server = HttpServer::new(String::from("127.0.0.1"), String::from("8443"), Vec::new(), None);
    /// ```
    #[cfg(not(feature = "threading"))]
    pub fn new(
        addr: String,
        port: String,
        handlers: Vec<(HttpMethod, String, HttpHandlerFn)>,
        default_handler: Option<HttpHandlerFn>,
    ) -> Result<Self, std::io::Error> {
        let default_handler_defined = match default_handler {
            Some(default_handler_defined) => default_handler_defined,
            None => Box::new(|_: &_| {
                HttpResponse::new(
                    String::from("1.1"),
                    http_base::http::http_structs::HttpStatus::NotImplemented,
                    None,
                    None,
                )
            }),
        };
        Ok(Self {
            listener: TcpListener::bind(format!("{addr}:{port}"))?,
            handlers,
            default_handler: default_handler_defined,
        })
    }
    /// Creates new instance of HttpServer
    /// Examples:
    /// ```rust
    /// use http_serv::http::server::HttpServer;
    ///
    /// // If num_cpus is enabled, threads can be set as the third arg. If no number is supplied, num_cpus will assume corecount * 3
    /// #[cfg(feature = "num_cpus")]
    /// let server = HttpServer::new(String::from("127.0.0.1"), String::from("8443"), Some(10), Vec::new(), None).unwrap();
    ///
    /// // If num_cpus is not enabled, thread count has to be specified
    /// #[cfg(not(feature = "num_cpus"))]
    /// let server = HttpServer::new(String::from("127.0.0.1"), String::from("8443"), 10, Vec::new(), None).unwrap();
    /// ```
    #[cfg(feature = "threading")]
    pub fn new(
        addr: String,
        port: String,
        #[cfg(feature = "num_cpus")] threads: Option<u32>,
        #[cfg(not(feature = "num_cpus"))] threads: u32,
        handlers: Vec<(HttpMethod, String, HttpHandlerFn)>,
        default_handler: Option<HttpHandlerFn>,
    ) -> Result<Self, std::io::Error> {
        #[cfg(feature = "num_cpus")]
        let threads = match threads {
            Some(threads) => threads,
            None => (num_cpus::get() as u32) * 3,
        };
        let default_handler_defined = match default_handler {
            Some(default_handler_defined) => default_handler_defined,
            None => Box::new(|_: &_| {
                HttpResponse::new(
                    String::from("1.1"),
                    http_base::http::http_structs::HttpStatus::NotImplemented,
                    None,
                    None,
                )
            }),
        };
        Ok(Self {
            listener: TcpListener::bind(format!("{addr}:{port}"))?,
            threads,
            handlers,
            default_handler: default_handler_defined,
        })
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
                    log::info!(
                        "[{}]: {}",
                        stream.peer_addr().unwrap_or(std::net::SocketAddr::new(
                            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                            0000
                        )),
                        http_request.http_headers.path
                    );
                    scope.execute(|| {
                        match handle_connection(
                            stream,
                            http_request,
                            &self.handlers,
                            &self.default_handler,
                        ) {
                            Ok(_) => {}
                            Err(_err) => {
                                #[cfg(feature = "log")]
                                log::error!("Encountered error handling connection: {_err}");
                            }
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
            log::info!(
                "[{}]: {}",
                stream.peer_addr().unwrap_or(std::net::SocketAddr::new(
                    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                    0000
                )),
                http_request.http_headers.path
            );
            match handle_connection(stream, http_request, &self.handlers, &self.default_handler) {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(feature = "log")]
                    log::error!("Encountered error handling connection: {_err}");
                }
            };
        }

        Ok(())
    }

    /// Adds a get method handler to the server
    /// Example:
    /// ```rust
    /// use http_serv::{HttpData, HttpResponse, HttpRequest, http_server::server::HttpServer};
    /// 
    /// let mut server = HttpServer::new("0.0.0.0".to_string(), "8443".to_string(), Vec::new(), None).unwrap();
    /// // :tag in a path will be used as route parameter
    /// server.get("/:uri".to_owned(), Box::new(|request: &HttpRequest| {
    ///     let mut resp = HttpResponse::default();
    ///     resp.data = Some(HttpData::Bytes(format!("{:#?}", request).as_bytes().to_vec()));
    ///     return resp;
    /// }));
    /// ```
    pub fn get(&mut self, path: String, exec: HttpHandlerFn) -> &mut Self {
        #[cfg(feature = "log")]
        log::debug!("Adding GET {path}");
        self.handlers.push((HttpMethod::GET, path, Box::new(exec)));
        self
    }

    /// Adds a put method handler to the server
    /// Example:
    /// ```rust
    /// use http_serv::{HttpData, HttpResponse, HttpRequest, http_server::server::HttpServer};
    /// 
    /// let mut server = HttpServer::new("0.0.0.0".to_string(), "8443".to_string(), Vec::new(), None).unwrap();
    /// // :tag in a path will be used as route parameter
    /// server.put("/:uri".to_owned(), Box::new(|request: &HttpRequest| {
    ///     let mut resp = HttpResponse::default();
    ///     resp.data = Some(HttpData::Bytes(format!("{:#?}", request).as_bytes().to_vec()));
    ///     return resp;
    /// }));
    /// ```
    pub fn post(&mut self, path: String, exec: HttpHandlerFn) -> &mut Self {
        #[cfg(feature = "log")]
        log::debug!("Adding POST {path}");
        self.handlers.push((HttpMethod::POST, path, Box::new(exec)));
        self
    }

    /// Adds a post method handler to the server
    /// Example:
    /// ```rust
    /// use http_serv::{HttpData, HttpResponse, HttpRequest, http_server::server::HttpServer};
    /// 
    /// let mut server = HttpServer::new("0.0.0.0".to_string(), "8443".to_string(), Vec::new(), None).unwrap();
    /// // :tag in a path will be used as route parameter
    /// server.post("/:uri".to_owned(), Box::new(|request: &HttpRequest| {
    ///     let mut resp = HttpResponse::default();
    ///     resp.data = Some(HttpData::Bytes(format!("{:#?}", request).as_bytes().to_vec()));
    ///     return resp;
    /// }));
    /// ```
    pub fn put(&mut self, path: String, exec: HttpHandlerFn) -> &mut Self {
        #[cfg(feature = "log")]
        log::debug!("Adding PUT {path}");
        self.handlers.push((HttpMethod::PUT, path, Box::new(exec)));
        self
    }

    /// Adds a delete method handler to the server
    /// Example:
    /// ```rust
    /// use http_serv::{HttpData, HttpResponse, HttpRequest, http_server::server::HttpServer};
    /// 
    /// let mut server = HttpServer::new("0.0.0.0".to_string(), "8443".to_string(), Vec::new(), None).unwrap();
    /// // :tag in a path will be used as route parameter
    /// server.delete("/:uri".to_owned(), Box::new(|request: &HttpRequest| {
    ///     let mut resp = HttpResponse::default();
    ///     resp.data = Some(HttpData::Bytes(format!("{:#?}", request).as_bytes().to_vec()));
    ///     return resp;
    /// }));
    /// ```
    pub fn delete(&mut self, path: String, exec: HttpHandlerFn) -> &mut Self {
        #[cfg(feature = "log")]
        log::debug!("Adding DELETE {path}");
        self.handlers
            .push((HttpMethod::DELETE, path, Box::new(exec)));
        self
    }

    /// Adds a default handler to the server
    /// Example:
    /// ```rust
    /// use http_serv::{HttpData, HttpResponse, HttpRequest, http_server::server::HttpServer};
    /// 
    /// let mut server = HttpServer::new("0.0.0.0".to_string(), "8443".to_string(), Vec::new(), None).unwrap();
    ///
    /// server.default(Box::new(|request: &HttpRequest| {
    ///     let mut resp = HttpResponse::default();
    ///     resp.data = Some(HttpData::Bytes(format!("{:#?}", request).as_bytes().to_vec()));
    ///     return resp;
    /// }));
    /// ```
    pub fn default(&mut self, exec: HttpHandlerFn) -> &mut Self {
        self.default_handler = exec;
        self
    }
}

fn handle_connection(
    mut stream: TcpStream,
    mut http_request: HttpRequest,
    handlers: &Vec<(HttpMethod, String, HttpHandlerFn)>,
    default_handler: &HttpHandlerFn,
) -> std::io::Result<()> {
    //let mut http_request = http_request.clone();
    let mut found_handler = false;
    // checks which function to run
    for handler in handlers {
        let mut route_params = Vec::new();
        let split_path = http_request.http_headers.path.split_once('?');

        // split route and query params and parse query
        let (route_params_unparsed, query_params): (String, Option<Vec<(String, String)>>) =
            match split_path {
                Some((rpu, qpu)) => {
                    // parse query params
                    (
                        rpu.to_owned(),
                        Some(http_request.query_params_from_string(qpu.to_owned())),
                    )
                }
                None => (http_request.http_headers.path.clone(), None),
            };

        http_request.query_params = query_params;

        // splits paths at /
        let received_parts = route_params_unparsed.split('/').collect::<Vec<&str>>();
        let defined_parts = handler.1.split('/').collect::<Vec<&str>>();

        for (idx_received, (received_section, defined_section)) in received_parts.iter().zip(defined_parts.iter()).enumerate() {
            if defined_section.starts_with(':') {
                // handle treat the rest of the path as single param
                if defined_section.ends_with('*') {
                    found_handler = true;
                    let paramcontent = received_parts[idx_received..].join("/");
                    route_params.push((
                        defined_section.to_owned().to_owned(),
                        paramcontent
                        //paramcontent.trim_end_matches('/').to_owned(),
                    ));
                    break;
                // handle single param
                } else {
                    route_params.push((
                        defined_section.to_owned().to_owned(),
                        received_section.to_owned().to_owned(),
                    ));
                }
                continue;
            }
            // does not match, break from inner loop
            if received_section != defined_section {
                break;
            }
        }

        // if there are parameters, add them to the request
        if !route_params.is_empty() {
            http_request.route_params = Some(route_params);
        }

        if handler.0 == http_request.http_headers.method {
            found_handler = true;
            #[cfg(feature = "log")]
            log::debug!(
                "Using handler {} for {} from {}",
                handler.1,
                http_request.http_headers.path,
                stream.peer_addr().unwrap_or(std::net::SocketAddr::new(
                    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                    0000
                ))
            );

            handle_closure(&mut stream, &http_request, &handler.2)?;
        }
    }
    if !found_handler {
        #[cfg(feature = "log")]
        log::warn!("Could not find handler, using default");

        handle_closure(&mut stream, &http_request, default_handler)?;
    }
    Ok(())
}

fn handle_closure(
    stream: &mut TcpStream,
    request: &HttpRequest,
    exec: &HttpHandlerFn,
) -> std::io::Result<()> {
    let response = exec(request);
    stream.write_all(response.to_headers().join("\r\n").as_bytes())?;
    match response.data {
        Some(data) => match data {
            http_base::http::http_structs::HttpData::Bytes(vec) => stream.write_all(&vec),
            http_base::http::http_structs::HttpData::Stream(mut read) => {
                match io::copy(&mut read.0, stream) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err),
                }
            }
        },
        None => Ok(()),
    }
}
