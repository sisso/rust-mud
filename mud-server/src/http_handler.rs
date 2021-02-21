///
/// Handle http request/response into game requests/responses
///
use http_server::{HttpMethod, HttpRequest, HttpResponse, HttpStatus};
use mud_domain::controller::Request;
use mud_domain::errors::Result;
use mud_domain::game::Game;
use serde_json::json;

pub struct HttpHandler {}

impl HttpHandler {
    pub fn new() -> Self {
        HttpHandler {}
    }
}

pub fn handle_requests(game: &mut Game, requests: Vec<HttpRequest>) -> Vec<HttpResponse> {
    requests
        .into_iter()
        .map(|http_request| {
            let request = match http_request {
                _ => Request::GetObjects,
            };

            match game.handle_request(&request) {
                _ => HttpResponse {
                    request: http_request,
                    status: HttpStatus::Error,
                    body: Some(json!({"error": "not implemented"})),
                },
            }
        })
        .collect()
}
