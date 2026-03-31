use domain::{
    errors::errors::AppResult,
    ports::ports_out::server::callback_server::{
        CallbackHandle, CallbackResponse, CallbackServer, CallbackServerError,
    },
};
use tiny_http::{Response, Server};
use tracing::{debug, error, info, warn};

pub struct TinyHttpCallbackServer {
    addr: String,
    path: String,
}

impl TinyHttpCallbackServer {
    pub fn new(addr: String, path: String) -> Self {
        Self { addr, path }
    }
}

impl CallbackServer for TinyHttpCallbackServer {
    fn start(&self) -> AppResult<Box<dyn CallbackHandle>> {
        info!(address = %self.addr, path = %self.path, "Starting callback server");
        let server = Server::http(&self.addr).map_err(|e| {
            error!(error = %e, "Failed to start callback server");
            CallbackServerError::StartFailed(anyhow::anyhow!("{e}"))
        })?;

        Ok(Box::new(TinyHttpCallbackHandle {
            server,
            path: self.path.clone(),
        }))
    }
}

pub struct TinyHttpCallbackHandle {
    server: Server,
    path: String,
}

impl TinyHttpCallbackHandle {
    fn extract_params(url: &str, path: &str) -> Option<(String, String)> {
        if !url.starts_with(path) {
            return None;
        }
        let query = url.split('?').nth(1)?;
        let mut code = None;
        let mut state = None;
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            match kv.next() {
                Some("code") => code = kv.next().map(String::from),
                Some("state") => state = kv.next().map(String::from),
                _ => {}
            }
        }
        Some((code?, state.unwrap_or_default()))
    }
}

impl CallbackHandle for TinyHttpCallbackHandle {
    fn wait_for_callback(&self) -> AppResult<CallbackResponse> {
        info!("Waiting for OAuth callback request");
        loop {
            let request = self.server.recv().map_err(|e| {
                error!(error = %e, "Failed to receive callback request");
                CallbackServerError::ReceiveFailed(anyhow::Error::from(e))
            })?;

            if let Some((code, state)) = Self::extract_params(request.url(), &self.path) {
                debug!("Received OAuth callback with code/state");
                let response =
                    Response::from_string(include_str!("../../../resources/auth_success.html"))
                        .with_header(
                            "Content-Type: text/html"
                                .parse::<tiny_http::Header>()
                                .unwrap(),
                        );
                request.respond(response).ok();
                info!("OAuth callback processed");
                return Ok(CallbackResponse { code, state });
            }

            warn!(url = request.url(), "Unexpected callback request path");
            let response = Response::from_string("Not found").with_status_code(404);
            request.respond(response).ok();
        }
    }
}
