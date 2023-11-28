use std::{net::{TcpStream, SocketAddr}, io::{BufReader, BufRead, Read, Error}};
/// HttpHeaders stores the HttpMethod, the Path and the Protocol used
#[derive(Debug, Clone)]
pub struct HttpHeaders {
    /// Stores the http method
    pub method: HttpMethod,
    /// Stores the requested path
    pub path: String,
    /// Stores the protocol used
    pub protocol: String
}

impl HttpHeaders {
    /// Creates new instance of HttpHeaders
    /// Example:
    /// ```rust
    /// use http_serv::http::http_structs::{HttpHeaders, HttpMethod};
    /// 
    /// let headers = HttpHeaders::new(HttpMethod::GET, String::from("/"), String::from("1.1"));
    /// ```
    pub fn new(method: HttpMethod, path: String, protocol: String) -> Self {
        Self { method, path, protocol }
    }

    /// Creates new instance of HttpHeaders from String
    /// Example:
    /// ```rust
    /// use http_serv::http::http_structs::HttpHeaders;
    /// 
    /// let headers_raw = String::from("GET / HTTP/1.1");
    /// let headers_parsed = HttpHeaders::from_line(headers_raw).unwrap();
    /// ```
    pub fn from_line(headerline: String) -> Result<Self, String> {
        let mut methodline = headerline.split(' ').collect::<Vec<&str>>();
        // resolve elements from header line
        let protocol = match methodline.pop() {
            Some(protocol) => protocol,
            None => {
                #[cfg(feature = "log")]
                log::trace!("Could not fetch protocol from headers");
                return Err(String::from("Could not fetch protocol from headers"))
            }
        };
        let path = match methodline.pop() {
            Some(path) => path,
            None => {
                #[cfg(feature = "log")]
                log::trace!("Could not fetch path from headers");
                return Err(String::from("Could not fetch path from headers"));
            },
        };
        let method = match methodline.pop() {
            Some("GET") => HttpMethod::GET,
            Some("PUT") => HttpMethod::PUT,
            Some("POST") => HttpMethod::POST,
            Some("DELETE") => HttpMethod::DELETE,
            Some(err) => {
                #[cfg(feature = "log")]
                log::trace!("Failed to fetch method, got: {err}");
                return Err(format!("Failed to fetch method, got: {err}"));
            },
            None => {
                #[cfg(feature = "log")]
                log::trace!("Failed to build headers, got no method");
                return Err(String::from("Failed to fetch method, got no method"));
            }
        };
        Ok(Self::new(method, path.to_string(), protocol.to_string()))
    }
}

/// HttpRequest stores the requests headers, request body, route and query parameters 
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// Client IP
    pub client_ip: SocketAddr,
    /// The http headers
    pub http_headers: HttpHeaders,
    /// Extra headers like Content-Length etc.
    pub extra_headers: Vec<(String, String)>,
    /// The http request's body as HttpData struct
    pub data: Option<HttpData>,
    /// The http requests route parameters
    pub route_params: Option<Vec<(String, String)>>,
    /// The http requests query parameters
    pub query_params: Option<Vec<(String, String)>>,
}

impl HttpRequest {
    /// Creates new instance of HttpRequest
    /// Example:
    /// ```rust
    /// use http_serv::http::http_structs::{HttpHeaders, HttpRequest};
    /// 
    /// let headers = HttpHeaders::from_line(String::from("GET / HTTP/1.1")).unwrap();
    /// let request = HttpRequest::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080), headers, Vec::new(), None, None, None);
    /// ```
    pub fn new(client_ip: SocketAddr, http_headers: HttpHeaders, extra_headers: Vec<(String, String)>, data: Option<HttpData>, route_params: Option<Vec<(String, String)>>, query_params: Option<Vec<(String, String)>>) -> Self {
        Self { client_ip, http_headers, extra_headers, data, route_params, query_params }
    }

    /// Tries to fetch extra headers from request by key
    /// ```rust
    /// use http_serv::http::http_structs::{HttpHeaders, HttpRequest};
    /// 
    /// let headers = HttpHeaders::from_line(String::from("GET / HTTP/1.1")).unwrap();
    /// let request = HttpRequest::new(headers, vec![(String::from("Content-Length"), String::from("5"))], None, None, None);
    /// let content_length = request.get_extra_header(String::from("Content-Length")).unwrap();
    /// ```
    pub fn get_extra_header(&self, header_name: String) -> Option<String> {
        for header in self.extra_headers.clone() {
            if header.0 == header_name {
                return Some(header.1);
            }
        }
        None
    }

    /// Tries to fetch route parameter from called route
    /// ```rust
    /// use http_serv::http::http_structs::{HttpHeaders, HttpRequest};
    /// 
    /// let headers = HttpHeaders::from_line(String::from("GET /test/value HTTP/1.1")).unwrap();
    /// let request = HttpRequest::new(headers, Vec::new(), None, Some(vec![(String::from(":var"), String::from("value"))]), None);
    /// let var_value = request.get_route_param(String::from(":var")).unwrap();
    /// ```
    pub fn get_route_param(&self, param_name: String) -> Option<String> {
        if let Some(route_params) = self.route_params.clone() {
            for route_param in route_params {
                if route_param.0 == param_name{
                    return Some(route_param.1);
                }
            }
        }
        None
    }

    /// Tries to fetch query parameter from called route
    /// ```rust
    /// use http_serv::http::http_structs::{HttpHeaders, HttpRequest};
    /// 
    /// let headers = HttpHeaders::from_line(String::from("GET /test?var=value HTTP/1.1")).unwrap();
    /// let request = HttpRequest::new(headers, Vec::new(), None, None, Some(vec![(String::from(":var"), String::from("value"))]));
    /// let var_value = request.get_query_param(String::from(":var")).unwrap();
    /// ```
    pub fn get_query_param(&self, param_name: String) -> Option<String> {
        if let Some(query_params) = self.query_params.clone() {
            for query_param in query_params {
                if query_param.0 == param_name{
                    return Some(query_param.1);
                }
            }
        }
        None
    }

    /// Tries to create a HttpRequest from an incomming TcpStream
    /// Example:
    /// ```ignore
    /// use std::net::{TcpListener, TcpStream};
    /// use http_serv::http::http_structs::HttpRequest;
    /// 
    /// let listener = TcpListener::bind(format!("127.0.0.1:9999")).unwrap();
    /// for stream in listener.incoming() {
    ///     let mut stream = stream.unwrap();
    ///     let request = HttpRequest::from_stream(&mut stream).unwrap();
    /// }
    /// ```
    pub fn from_stream(stream: &mut TcpStream) -> Result<Self, std::io::Error> {
        let mut buf_reader = BufReader::new(&*stream);
        let mut lines = Vec::new();
        let client_ip = stream.peer_addr()?;
        // read until no bytes are being read
        loop {
            let mut line = String::new();
            match buf_reader.read_line(&mut line) {
                Ok(0) => break, // End of stream
                Ok(_) => {
                    if line.trim().is_empty() {
                        break; // Empty line signifies end of headers
                    }
                    lines.push(line.trim().to_owned());
                }
                Err(err) => {
                    #[cfg(feature = "log")]
                    log::trace!("Error reading headers from stream: {err}");
                    return Err(err);
                }, // Error reading line
            }
        }
        let mut lines = lines.iter();
        let http_headers = match HttpHeaders::from_line(lines.next().unwrap_or(&String::new()).clone()) {
            Ok(http_headers) => http_headers,
            Err(err) => {
                #[cfg(feature = "log")]
                log::trace!("Failed to build headers: {err}");
                return Err(Error::new(std::io::ErrorKind::InvalidData, err));
            },
        };
        let mut extra_headers = Vec::new();
        let mut _data = None;

        // split header lines at : and create tuples
        for line in lines {
            if line.is_empty() {
                break;
            }
            match line.split_once(':') {
                Some((key , val )) => extra_headers.push((key.trim(), val.trim())),
                None => {
                    #[cfg(feature = "log")]
                    log::warn!("Invalid header: {line}");
                }
            }
        }

        // check if there is http data to be fetched
        for extra_header in &extra_headers {
            if let ("Content-Length", length) = extra_header {
                let length = match length.parse() {
                    Ok(length) => length,
                    Err(_) => {
                        #[cfg(feature = "log")]
                        log::warn!("Failed to parse content length from {length}. Setting to 0 and ignoring body");
                        continue;
                    },
                };
                let mut buffer = vec![0; length];
                match buf_reader.read_exact(&mut buffer) {
                    Ok(_) => _data = Some(HttpData::new(buffer)),
                    Err(err) => {
                        #[cfg(feature = "log")]
                        log::trace!("Error reading request body");
                        return Err(err);
                    },
                }
            }
        }

        Ok(Self::new(client_ip, http_headers, extra_headers.iter().map(|(key, val)| {
            (key.to_owned().to_owned(), val.to_owned().to_owned())
        }).collect::<Vec<(String, String)>>(), _data, None, None))
    }

    /// Creates query params from string
    /// Example:
    /// ```rust
    /// use http_serv::http::http_structs::{HttpHeaders, HttpRequest};
    /// 
    /// let query_string = String::from("/route?param1=val1&param2=val2");
    /// let split = query_string.split_once('?').unwrap().1;
    /// let headers = HttpHeaders::from_line(String::from("GET /test?var=value HTTP/1.1")).unwrap();
    /// let mut request = HttpRequest::new(headers, Vec::new(), None, None, None);
    /// request.query_params = Some(request.query_params_from_string(query_string));
    /// ```
    pub fn query_params_from_string(&mut self, params: String) -> Vec<(String, String)>{
        let split = params.split('&').collect::<Vec<&str>>();
        let mut key_val = Vec::new();
        for split_elem in split {
            match split_elem.split_once('=') {
                Some((key, val)) => key_val.push((key.to_owned(), val.to_owned())),
                None => {
                    #[cfg(feature = "log")]
                    log::warn!("Invalid query pair: {split_elem}");
                },
            }
        }
        key_val
    }
}

/// HttpResponse stores the http version, the status code (eg. 200 - OK), extra headers and eventual binary data
#[derive(Debug)]
pub struct HttpResponse {
    /// Http version eg. 1.1
    pub http_ver: String,
    /// Http status like OK, NotFound etc.
    pub status: HttpStatus,
    /// Extra headers like Content-Length: 10
    pub extra_headers: Option<Vec<(String, String)>>,
    /// Binary body data like images or files 
    pub data: Option<HttpData>
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self { http_ver: "1.1".to_string(), status: HttpStatus::Ok, extra_headers: None, data: None }
    }
}

impl HttpResponse {
    /// Creates new instance of HttpResponse
    /// Example:
    /// ```rust
    /// use http_serv::http::http_structs::{HttpResponse, HttpStatus};
    /// 
    /// let response = HttpResponse::new(String::from("1.1"), HttpStatus::Ok, None, None);
    /// ```
    pub fn new(http_ver: String, status: HttpStatus, extra_headers: Option<Vec<(String, String)>>, data: Option<HttpData>) -> Self {
        Self { http_ver, status, extra_headers, data }
    }
    
    /// Tries to fetch extra header by key
    /// Example:
    /// ```rust
    /// use http_serv::http::http_structs::{HttpResponse, HttpStatus};
    /// 
    /// let response = HttpResponse::new(String::from("1.1"), HttpStatus::Ok, Some(vec![(String::from("Content-Length"), String::from("5"))]), None);
    /// let content_length = response.get_extra_header(String::from("Content-Length")).unwrap();
    /// ```
    pub fn get_extra_header(&self, header_name: String) -> Option<String> {
        if let Some(extra_headers) = self.extra_headers.clone() {
            for header in extra_headers {
                if header.0 == header_name {
                    return Some(header.1);
                }
            }
        }
        None
    }

    /// Creates a vec of lines which contain the http header, and extra headers. Autodetects eventual binary data present and sets content-length accordingly
    /// ```rust
    /// use http_serv::http::http_structs::{HttpResponse, HttpStatus};
    /// 
    /// let response = HttpResponse::new(String::from("1.1"), HttpStatus::Ok, Some(vec![(String::from("Content-Length"), String::from("5"))]), None);
    /// let headers = response.to_headers();
    /// ```
    pub fn to_headers(&self) -> Vec<String> {
        let mut headers = Vec::new();
        headers.push(format!("HTTP/{} {}", self.http_ver, self.status_to_header()));
        match &self.data {
            Some(data) => headers.push(data.to_header()),
            None => {}
        }
        if let Some(extra_headers) = self.extra_headers.clone() {
            for extra_header in extra_headers {
                headers.push(format!("{}: {}", extra_header.0, extra_header.1))
            }
        }
        
        headers.push("\r\n".to_string());
        headers
    }

    fn status_to_header(&self) -> String {
        match self.status {
            HttpStatus::Continue => String::from("100 Continue"),
            HttpStatus::SwitchingProtocols => String::from("101 Switching Protocols"),
            HttpStatus::Processing => String::from("102 Processing"),
            HttpStatus::EarlyHints => String::from("103 Early Hints"),
            HttpStatus::Ok => String::from("200 OK"),
            HttpStatus::Created => String::from("201 Created"),
            HttpStatus::Accepted => String::from("202 Accepted"),
            HttpStatus::NonAuthorativeInformation => String::from("203 Non-Authoritative Information"),
            HttpStatus::NoContent => String::from("204 No Content"),
            HttpStatus::ResetContent => String::from("205 Reset Content"),
            HttpStatus::PartialContent => String::from("206 Partial Content"),
            HttpStatus::MultiStatus => String::from("207 Multi-Status"),
            HttpStatus::AlreadyReported => String::from("208 Already Reported"),
            HttpStatus::IMUsed => String::from("226 IM Used"),
            HttpStatus::MultipleChoices => String::from("300 Multiple Choices"),
            HttpStatus::MovedPermanently => String::from("301 Moved Permanently"),
            HttpStatus::Found => String::from("302 Found"),
            HttpStatus::SeeOther => String::from("303 See Other"),
            HttpStatus::NotModified => String::from("304 Not Modified"),
            HttpStatus::UseProxy => String::from("305 Use Proxy"),
            HttpStatus::Unused => String::from("306 Unused"),
            HttpStatus::TemporaryRedirect => String::from("307 Temporary Redirect"),
            HttpStatus::PermanentRedirect => String::from("308 Permanent Redirect"),
            HttpStatus::BadRequest => String::from("400 Bad Request"),
            HttpStatus::Unauthorized => String::from("401 Unauthorized"),
            HttpStatus::PaymentRequired => String::from("402 Payment Required"),
            HttpStatus::Forbidden => String::from("403 Forbidden"),
            HttpStatus::NotFound => String::from("404 Not Found"),
            HttpStatus::MethodNotAllowed => String::from("405 Method Not Allowed"),
            HttpStatus::NotAcceptable => String::from("406 Not Acceptable"),
            HttpStatus::ProxyAuthenticationRequired => String::from("407 Proxy Authentication Required"),
            HttpStatus::RequestTimeout => String::from("408 Request Timeout"),
            HttpStatus::Conflict => String::from("409 Conflict"),
            HttpStatus::Gone => String::from("410 Gone"),
            HttpStatus::LengthRequired => String::from("411 Length Required"),
            HttpStatus::PreconditionFailed => String::from("412 Precondition Failed"),
            HttpStatus::PayloadTooLarge => String::from("413 Payload Too Large"),
            HttpStatus::URITooLong => String::from("414 URI Too Long"),
            HttpStatus::UnsupportedMediaType => String::from("415 Unsupported Media Type"),
            HttpStatus::RangeNotSatisfiable => String::from("416 Range Not Satisfiable"),
            HttpStatus::ExpectationFailed => String::from("417 Expectation Failed"),
            HttpStatus::IMATeapot => String::from("418 I'm a teapot"),
            HttpStatus::MisdirectedRequest => String::from("421 Misdirected Request"),
            HttpStatus::UnprocessableContent => String::from("422 Unprocessable Entity"),
            HttpStatus::Locked => String::from("423 Locked"),
            HttpStatus::FailedDependency => String::from("424 Failed Dependency"),
            HttpStatus::TooEarly => String::from("425 Too Early"),
            HttpStatus::UpgradeRequired => String::from("426 Upgrade Required"),
            HttpStatus::PreconditionRequired => String::from("428 Precondition Required"),
            HttpStatus::TooManyRequests => String::from("429 Too Many Requests"),
            HttpStatus::RequestsHeaderFieldsTooLarge => String::from("431 Request Header Fields Too Large"),
            HttpStatus::UnavailableForLegalReasons => String::from("451 Unavailable For Legal Reasons"),
            HttpStatus::InternalServerError => String::from("500 Internal Server Error"),
            HttpStatus::NotImplemented => String::from("501 Not Implemented"),
            HttpStatus::BadGateway => String::from("502 Bad Gateway"),
            HttpStatus::ServiceUnavailable => String::from("503 Service Unavailable"),
            HttpStatus::GatewayTimeout => String::from("504 Gateway Timeout"),
            HttpStatus::HTTPVersionNotSupported => String::from("505 HTTP Version Not Supported"),
            HttpStatus::VariantAlsoNegotiates => String::from("506 Variant Also Negotiates"),
            HttpStatus::InsufficientStorage => String::from("507 Insufficient Storage"),
            HttpStatus::LoopDetected => String::from("508 Loop Detected"),
            HttpStatus::NotExtended => String::from("510 Not Extended"),
            HttpStatus::NetworkAuthenticationRequired => String::from("511 Network Authentication Required"),
         }         
    }
}

/// Represents supported http methods
#[derive(Debug, PartialEq, Clone)]
pub enum HttpMethod {
    /// Post data to server
    POST,
    /// Updated data on server
    PUT,
    /// Fetch data from server
    GET,
    /// Delete data from server
    DELETE
}

/// Represents http status codes with their keywords
#[derive(Debug, num_enum::TryFromPrimitive)]
#[repr(u64)]
// Status codes + description from https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
/// Represents the status of an HTTP response.
pub enum HttpStatus {
    /// Continue processing request.
    Continue = 100,
    /// Switching to new protocol.
    SwitchingProtocols = 101,
    /// Processing, please wait.
    Processing = 102,
    /// Early hints for the request.
    EarlyHints = 103,
    /// Request succeeded.
    Ok = 200,
    /// Resource created successfully.
    Created = 201,
    /// Request accepted for processing.
    Accepted = 202,
    /// Information not authoritative.
    NonAuthorativeInformation = 203,
    /// No content to send for this request.
    NoContent = 204,
    /// Reset content for this request.
    ResetContent = 205,
    /// Partial content for this request.
    PartialContent = 206,
    /// Multiple statuses for this request.
    MultiStatus = 207,
    /// Already reported for this request.
    AlreadyReported = 208,
    /// IM used for this request.
    IMUsed = 226,
    /// Multiple choices for this request.
    MultipleChoices = 300,
    /// Moved permanently to a new URL.
    MovedPermanently = 301,
    /// Found, but further action required.
    Found = 302,
    /// See other URL for this request.
    SeeOther = 303,
    /// Not modified since last request.
    NotModified = 304,
    /// Use proxy for this request.
    UseProxy = 305,
    /// Unused status code.
    Unused = 306,
    /// Temporary redirect to a new URL.
    TemporaryRedirect = 307,
    /// Permanent redirect to a new URL.
    PermanentRedirect = 308,
    /// Bad request syntax or unsupported method.
    BadRequest = 400,
    /// Unauthorized request.
    Unauthorized = 401,
    /// Payment required for this request.
    PaymentRequired = 402,
    /// Forbidden request.
    Forbidden = 403,
    /// Resource not found.
    NotFound = 404,
    /// Method not supported for this resource.
    MethodNotAllowed = 405,
    /// Request not acceptable.
    NotAcceptable = 406,
    /// Proxy authentication required.
    ProxyAuthenticationRequired = 407,
    /// Request timeout.
    RequestTimeout = 408,
    /// Request conflict.
    Conflict = 409,
    /// Resource is gone.
    Gone = 410,
    /// Length required for this request.
    LengthRequired = 411,
    /// Precondition failed for this request.
    PreconditionFailed = 412,
    /// Payload too large for this request.
    PayloadTooLarge = 413,
    /// URI too long for this request.
    URITooLong = 414,
    /// Unsupported media type for this request.
    UnsupportedMediaType = 415,
    /// Range not satisfiable for this request.
    RangeNotSatisfiable = 416,
    /// Expectation failed for this request.
    ExpectationFailed = 417,
    /// IM a teapot for this request.
    IMATeapot = 418,
    /// Misdirected request.
    MisdirectedRequest = 421,
    /// Unprocessable content for this request.
    UnprocessableContent = 422,
    /// Locked for this request.
    Locked = 423,
    /// Failed dependency for this request.
    FailedDependency = 424,
    /// Too early for this request.
    TooEarly = 425,
    /// Upgrade required for this request.
    UpgradeRequired = 426,
    /// Precondition required for this request.
    PreconditionRequired = 428,
    /// Too many requests for this user.
    TooManyRequests = 429,
    /// Request header fields too large for this request.
    RequestsHeaderFieldsTooLarge = 431,
    /// Unavailable for legal reasons for this request.
    UnavailableForLegalReasons = 451,
    /// Internal server error.
    InternalServerError = 500,
    /// Not implemented for this request.
    NotImplemented = 501,
    /// Bad gateway for this request.
    BadGateway = 502,
    /// Service unavailable for this request.
    ServiceUnavailable = 503,
    /// Gateway timeout for this request.
    GatewayTimeout = 504,
    /// HTTP version not supported for this request.
    HTTPVersionNotSupported = 505,
    /// Variant also negotiates for this request.
    VariantAlsoNegotiates = 506,
    /// Insufficient storage for this request.
    InsufficientStorage = 507,
    /// Loop detected for this request.
    LoopDetected = 508,
    /// Not extended for this request.
    NotExtended = 510,
    /// Network authentication required for this request.
    NetworkAuthenticationRequired = 511,
  }  

/// Represents binary data present in the body of a http request
#[derive(Debug, Clone)]
pub struct HttpData {
    /// The data represented as a vector of bytes.
    pub data: Vec<u8>
}

impl HttpData {
    /// Creates new instance of HttpData from a vec of bytes
    /// Example:
    /// ```rust
    /// use http_serv::http::http_structs::HttpData;
    /// 
    /// let http_data = HttpData::new(vec![b't', b'e', b's', b't']);
    /// ```
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Fetches the length of the stored binary and sets the Content-Length header accordingly
    /// Example:
    /// ```rust
    /// use http_serv::http::http_structs::HttpData;
    /// 
    /// let http_data = HttpData::new(vec![b't', b'e', b's', b't']);
    /// let header = http_data.to_header();
    /// ```
    pub fn to_header(&self) -> String {
        format!("Content-Length: {}", self.data.len())
    }
}