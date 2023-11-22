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