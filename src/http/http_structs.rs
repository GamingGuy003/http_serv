pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub protocol: String,
    pub data: Option<HttpData>
}

pub struct HttpResponse {
    pub http_ver: String,
    pub status: HttpStatus,
    pub extra_headers: Option<Vec<(String, String)>>,
    pub data: Option<HttpData>
}

impl HttpResponse {
    pub fn new(http_ver: String, status: HttpStatus, extra_headers: Option<Vec<(String, String)>>, data: Option<HttpData>) -> Self {
        Self { http_ver, status, extra_headers, data }
    }
    
    pub fn response_to_headers(&self) -> Vec<String> {
        let mut headers = Vec::new();
        headers.push(format!("HTTP/{} {}", self.http_ver, self.resolve_status_to_header()));
        match &self.data {
            Some(data) => headers.push(data.data_to_header()),
            None => {}
        }
        headers.push("\r\n".to_string());
        headers
    }

    fn resolve_status_to_header(&self) -> String {
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

pub enum HttpMethod {
    POST,
    PUT,
    GET,
    DELETE
}

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
pub struct HttpData {
    data: Vec<u8>
}

impl HttpData {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn data_to_header(&self) -> String {
        format!("Content-Length: {}", self.data.len())
    }
}