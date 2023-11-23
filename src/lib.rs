mod http;

#[cfg(test)]
mod tests {
    use crate::http::http_structs::{HttpResponse, HttpData};
    use crate::http::http_structs::HttpStatus::{Ok, IMATeapot};
    use crate::http::server::HttpServer;

    #[test]
    pub fn response_headers() {
        let response = HttpResponse::new("2".to_string(), Ok, None, None);
        assert_eq!(response.to_headers().join("\r\n"), String::from("HTTP/2 200 OK\r\n\r\n"))
    }

    #[test]
    pub fn response_headers_with_payload() {
        let data = HttpData::new(vec![b't', b'e', b's', b't']);
        let response = HttpResponse::new("2".to_string(), IMATeapot, None, Some(data));
        assert_eq!(response.to_headers().join("\r\n"), String::from("HTTP/2 418 I'm a teapot\r\nContent-Length: 4\r\n\r\n"));
    }

    #[test]
    pub fn server_test() {
        let server = HttpServer::new("0.0.0.0".to_owned(), "8443".to_owned(), Vec::new());
        let mut server = server.unwrap();
        server.get("/".to_owned(), |request| {
            print!("{:#?}", request);
            let mut resp = HttpResponse::default();
            resp.data = Some(HttpData::new(format!("{:#?}", request).into_bytes()));
            println!("{:#?}", resp);
            resp
        });
        server.run_loop().unwrap();
    }
}
