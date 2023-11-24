pub mod http;

#[cfg(test)]
mod tests {
    use crate::http::http_structs::{HttpResponse, HttpData, HttpStatus};
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
    pub fn server_run() {
        #[cfg(feature = "log")]
        pretty_env_logger::init_timed();
        let mut server = HttpServer::new("0.0.0.0".to_string(), "8443".to_string(), 10, Vec::new()).unwrap();
        server.get("/:uri".to_owned(), |_| {
            std::thread::sleep(std::time::Duration::from_secs(15));
            let resp = HttpResponse::new("1.1".to_string(), HttpStatus::NotFound, Some(vec![("Location".to_owned(), "https://google.de".to_owned())]), None);
            resp
        });
        server.post("/_shorten/:uri".to_owned(), |request| {
            let mut resp = HttpResponse::default();
            resp.data = Some(HttpData::new(format!("{:#?}", request).as_bytes().to_vec()));
            resp
        });
        server.delete("/_delete/:uri".to_owned(), |request| {
            let mut resp = HttpResponse::default();
            resp.data = Some(HttpData::new(format!("{:#?}", request).as_bytes().to_vec()));
            resp
        });
        server.get("/_info/:uri".to_owned(), |request| {
            let mut resp = HttpResponse::default();
            resp.data = Some(HttpData::new(format!("{:#?}", request).as_bytes().to_vec()));
            resp
        });
        let _ = server.run_loop();
    }
}
