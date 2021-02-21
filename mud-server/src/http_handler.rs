///
/// Handle http request/response into game requests/responses
///
use http_server::{HttpMethod, HttpRequest, HttpResponse, HttpStatus};
use mud_domain::controller::{Request, Response};
use mud_domain::errors::Result;
use mud_domain::game::loader::dto::StaticId;
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
        .map(|http_request| handle_request(game, http_request))
        .collect()
}

fn handle_request(game: &mut Game, http_request: HttpRequest) -> HttpResponse {
    match route_path(http_request) {
        Ok((http_request, request)) => match game.handle_request(&request) {
            Ok(Response::GetObjects { objects }) => {
                http_request.into_success_body(json!({ "objects": objects }))
            }
            Ok(Response::GetObject { object }) => http_request.into_success_body(object),
            Err(err) => http_request.into_error_response(HttpStatus::Error, &format!("{}", err)),
        },
        Err(response) => response,
    }
}

fn route_path(
    http_request: HttpRequest,
) -> std::result::Result<(HttpRequest, Request), HttpResponse> {
    let paths: Vec<&str> = http_request.path.split("/").skip(1).collect();

    let request = match (&http_request.method, paths.as_slice()) {
        (HttpMethod::GET, ["objects"]) => Request::GetObjects,
        (HttpMethod::GET, ["objects", id_str]) => match id_str.parse::<u32>() {
            Ok(id) => Request::GetObj(id.into()),
            _ => {
                let error_msg = &format!("invalid id '{}'", id_str);
                return Err(http_request.into_error_response(HttpStatus::Invalid, error_msg));
            }
        },
        (HttpMethod::GET, ["prefabs"]) => Request::GetPrefabs,
        (HttpMethod::GET, ["prefabs", id_str]) => match id_str.parse::<u32>() {
            Ok(static_id) => Request::GetPrefab(StaticId(static_id)),
            _ => {
                let error_msg = &format!("invalid id '{}'", id_str);
                return Err(http_request.into_error_response(HttpStatus::Invalid, error_msg));
            }
        },
        _ => {
            return Err(http_request.into_error_response(HttpStatus::NotFound, "not found"));
        }
    };

    Ok((http_request, request))
}
