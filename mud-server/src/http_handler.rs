///
/// Handle http request/response into game requests/responses
///
use http_server::{HttpMethod, HttpRequest, HttpRequestId, HttpResponse, HttpStatus};
use mud_domain::controller::{Request, Response};
use mud_domain::errors::{Error, Result};
use mud_domain::game::loader::dto::StaticId;
use mud_domain::game::Game;
use serde_json::json;

pub fn handle_requests(game: &mut Game, requests: Vec<HttpRequest>) -> Vec<HttpResponse> {
    requests
        .into_iter()
        .map(|http_request| handle_request(game, &http_request))
        .collect()
}

fn handle_request(game: &mut Game, http_request: &HttpRequest) -> HttpResponse {
    match map_http_into_request(&http_request) {
        Ok(request) => {
            let resp = game.handle_request(&request);
            map_response_into_http(http_request.request_id, resp)
        }
        Err(response) => response,
    }
}

fn map_response_into_http(request_id: HttpRequestId, resp: Result<Response>) -> HttpResponse {
    match resp {
        Ok(Response::GetObjects { objects }) => {
            HttpResponse::new_success_body(request_id, json!({ "objects": objects }))
        }
        Ok(Response::GetObject { object }) => HttpResponse::new_success_body(request_id, object),
        Ok(Response::GetPrefabs { prefabs }) => {
            HttpResponse::new_success_body(request_id, json!({ "prefabs": prefabs }))
        }
        Ok(Response::GetPrefab { prefab }) => HttpResponse::new_success_body(request_id, prefab),
        Err(Error::NotFoundStaticId(_)) | Err(Error::NotFoundFailure) => HttpResponse::new_error(
            request_id,
            HttpStatus::NotFound,
            &format!("{}", "not found"),
        ),
        Err(err) => HttpResponse::new_error(request_id, HttpStatus::Error, &format!("{}", err)),
    }
}

fn map_http_into_request(http_request: &HttpRequest) -> std::result::Result<Request, HttpResponse> {
    let paths: Vec<&str> = http_request.path.split("/").skip(1).collect();

    let request = match (&http_request.method, paths.as_slice()) {
        (HttpMethod::GET, ["objects"]) => Request::GetObjects,
        (HttpMethod::GET, ["objects", id_str]) => match id_str.parse::<u32>() {
            Ok(id) => Request::GetObj(id.into()),
            _ => {
                let error_msg = &format!("invalid id '{}'", id_str);
                return Err(HttpResponse::new_error(
                    http_request.request_id,
                    HttpStatus::Invalid,
                    error_msg,
                ));
            }
        },
        (HttpMethod::GET, ["prefabs"]) => Request::GetPrefabs,
        (HttpMethod::GET, ["prefabs", id_str]) => match id_str.parse::<u32>() {
            Ok(static_id) => Request::GetPrefab(StaticId(static_id)),
            _ => {
                let error_msg = &format!("invalid id '{}'", id_str);
                return Err(HttpResponse::new_error(
                    http_request.request_id,
                    HttpStatus::Invalid,
                    error_msg,
                ));
            }
        },
        _ => {
            return Err(HttpResponse::new_error(
                http_request.request_id,
                HttpStatus::NotFound,
                "not found",
            ));
        }
    };

    Ok(request)
}
