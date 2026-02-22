use domain::{
    errors::errors::AppResult,
    ports::ports_out::server::callback_server::{CallbackServer, CallbackServerError},
};
use tiny_http::{Response, Server};

pub struct TinyHttpCallbackServer {
    addr: String,
    path: String,
}

impl TinyHttpCallbackServer {
    pub fn new(addr: String, path: String) -> Self {
        Self { addr, path }
    }

    fn extract_code(url: &str, path: &str) -> Option<String> {
        if !url.starts_with(path) {
            return None;
        }
        let query = url.split('?').nth(1)?;
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            if kv.next() == Some("code") {
                return kv.next().map(String::from);
            }
        }
        None
    }
}

impl CallbackServer for TinyHttpCallbackServer {
    fn wait_for_callback(&self) -> AppResult<String> {
        let server = Server::http(&self.addr)
            .map_err(|e| CallbackServerError::StartFailed(anyhow::anyhow!("{e}")))?;

        loop {
            let request = server.recv()
                .map_err(|e| CallbackServerError::ReceiveFailed(anyhow::Error::from(e)))?;

            if let Some(code) = Self::extract_code(request.url(), &self.path) {
                let response = Response::from_string(include_str!("../../../resources/auth_success.html"))
                    .with_header(
                        "Content-Type: text/html".parse::<tiny_http::Header>().unwrap(),
                    );
                request.respond(response).ok();
                return Ok(code);
            }

            let response = Response::from_string("Not found").with_status_code(404);
            request.respond(response).ok();
        }
    }
}
