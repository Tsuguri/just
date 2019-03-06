mod ws_handler;
pub use ws_handler::*;

use web_sys::WebSocket;
use web_sys::console;

pub struct ConnectionState {

    pub ws: Option<WebSocket>,
    pub lol: i32,
}

unsafe impl Send for ConnectionState {}

impl ConnectionState {
    pub fn new() -> Self {
        ConnectionState {
            ws: Option::None,
            lol: 0
        }
    }

    pub fn connect(&mut self, address: &str) -> Result<(), &'static str> {
       if let Option::Some(_) = self.ws {
           console::log_1(&"Unable to connect: connection already existing.".into());
           return Result::Err("Connection already existing");
       } 
       console::log_1(&"Connected".into());
       Result::Ok(())
    }
}