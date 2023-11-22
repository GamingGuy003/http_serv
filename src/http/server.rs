use std::net::TcpListener;

pub struct Server {
    socket: TcpListener,
    handlers: Vec<(String, Box<dyn Fn()>)>,
    
}