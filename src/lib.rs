mod http;

#[cfg(test)]
mod tests {
    use crate::http::http_structs::HttpResponse;
    use crate::http::http_structs::HttpStatus::Ok;

    #[test]
    pub fn response_creation() {
        let response = HttpResponse::new("2".to_string(), Ok, None, None);
    }
}
