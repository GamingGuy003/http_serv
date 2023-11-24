use std::{net::TcpStream, io::{BufReader, BufRead, Read}};

#[derive(Debug)]
pub struct HttpHeaders {
    pub method: HttpMethod,
    pub path: String,
    pub protocol: String
}

impl HttpHeaders {
    pub fn new(method: HttpMethod, path: String, protocol: String) -> Self {
        Self { method, path, protocol }
    }

    pub fn from_line(headerline: String) -> Result<Self, std::io::Error> {
        let mut methodline = headerline.split(' ').collect::<Vec<&str>>();
        // resolve elements from header line
        let protocol = methodline.pop().expect("Could not fetch protocol from headers");
        let path = methodline.pop().expect("Could not fetch path from headers");
        let method = match methodline.pop().expect("Could not fetch method from headers") {
            "GET" => HttpMethod::GET,
            "PUT" => HttpMethod::PUT,
            "POST" => HttpMethod::POST,
            "DELETE" => HttpMethod::DELETE,
            meth_err => panic!("Failed to resolve method, got {meth_err}")
        };
        Ok(Self::new(method, path.to_string(), protocol.to_string()))
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub http_headers: HttpHeaders,
    pub extra_headers: Vec<(String, String)>,
    pub data: Option<HttpData>,
    pub route_params: Option<Vec<(String, String)>>,
    pub query_params: Option<Vec<(String, String)>>,
}

impl HttpRequest {
    pub fn new(http_headers: HttpHeaders, extra_headers: Vec<(String, String)>, data: Option<HttpData>, route_params: Option<Vec<(String, String)>>, query_params: Option<Vec<(String, String)>>) -> Self {
        Self { http_headers, extra_headers, data, route_params, query_params }
    }

    pub fn from_stream(stream: &mut TcpStream) -> Result<Self, std::io::Error> {
        let mut buf_reader = BufReader::new(&*stream);
        let mut lines = Vec::new();
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
                    log::error!("Failed to read headers");
                    return Err(err);
                }, // Error reading line
            }
        }
        let mut lines = lines.iter();
        let http_headers = HttpHeaders::from_line(lines.next().expect("Failed to get header line").clone())?;
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
                let length = length.parse().expect("Failed to parse Content-Length");
                let mut buffer = vec![0; length];
                match buf_reader.read_exact(&mut buffer) {
                    Ok(_) => _data = Some(HttpData::new(buffer)),
                    Err(err) => {
                        #[cfg(feature = "log")]
                        log::error!("Error reading request body");
                        return Err(err);
                    },
                }
            }
        }

        Ok(Self::new(http_headers, extra_headers.iter().map(|(key, val)| {
            (key.to_owned().to_owned(), val.to_owned().to_owned())
        }).collect::<Vec<(String, String)>>(), _data, None, None))
    }

    pub fn query_params_from_string(&mut self, params: String) -> Vec<(String, String)>{
        let split = params.split('&').collect::<Vec<&str>>();
        let mut key_val = Vec::new();
        for split_elem in split {
            match split_elem.split_once('=') {
                Some((key, val)) => key_val.push((key.to_owned(), val.to_owned())),
                None => println!("Invalid key - value pair for query {split_elem}"),
            }
        }
        key_val
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub http_ver: String,
    pub status: HttpStatus,
    pub extra_headers: Option<Vec<(String, String)>>,
    pub data: Option<HttpData>
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self { http_ver: "1.1".to_string(), status: HttpStatus::Ok, extra_headers: None, data: None }
    }
}

impl HttpResponse {
    pub fn new(http_ver: String, status: HttpStatus, extra_headers: Option<Vec<(String, String)>>, data: Option<HttpData>) -> Self {
        Self { http_ver, status, extra_headers, data }
    }
    
    pub fn to_headers(&self) -> Vec<String> {
        let mut headers = Vec::new();
        headers.push(format!("HTTP/{} {}", self.http_ver, self.status_to_header()));
        match &self.data {
            Some(data) => headers.push(data.to_header()),
            None => {}
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

#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    POST,
    PUT,
    GET,
    DELETE
}

#[derive(Debug)]
// Status codes + description from https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
pub enum HttpStatus {
    Continue, // 100
    SwitchingProtocols, // 101
    Processing, // 102
    EarlyHints, // 103
    Ok, //200
    Created, // 201
    Accepted, // 202
    NonAuthorativeInformation, //203
    NoContent, // 204
    ResetContent, // 205
    PartialContent, // 206
    MultiStatus, // 207
    AlreadyReported, // 208
    IMUsed, // 226
    MultipleChoices, // 300
    MovedPermanently, // 301
    Found, // 302
    SeeOther, // 303
    NotModified, // 304
    UseProxy, // 305
    Unused, // 306
    TemporaryRedirect, // 307
    PermanentRedirect, // 308
    BadRequest, // 400
    Unauthorized, // 401
    PaymentRequired, // 402
    Forbidden, // 403
    NotFound, // 404
    MethodNotAllowed, // 405
    NotAcceptable, // 406
    ProxyAuthenticationRequired, // 407
    RequestTimeout, // 408
    Conflict, // 409
    Gone, // 410
    LengthRequired, // 411
    PreconditionFailed, // 412
    PayloadTooLarge, // 413
    URITooLong, // 414
    UnsupportedMediaType, // 415
    RangeNotSatisfiable, // 416
    ExpectationFailed, // 417
    IMATeapot, // 418
    MisdirectedRequest, // 421
    UnprocessableContent, // 422
    Locked, // 423
    FailedDependency, // 424
    TooEarly, // 425
    UpgradeRequired, // 426
    PreconditionRequired, // 428
    TooManyRequests, // 429
    RequestsHeaderFieldsTooLarge, // 431
    UnavailableForLegalReasons, // 451
    InternalServerError, // 500
    NotImplemented, // 501
    BadGateway, // 502
    ServiceUnavailable, // 503
    GatewayTimeout, // 504
    HTTPVersionNotSupported, // 505
    VariantAlsoNegotiates, // 506
    InsufficientStorage, // 507
    LoopDetected, // 508
    NotExtended, // 510
    NetworkAuthenticationRequired, // 511
}

#[derive(Debug)]
pub struct HttpData {
    pub data: Vec<u8>
}

impl HttpData {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn to_header(&self) -> String {
        format!("Content-Length: {}", self.data.len())
    }
}