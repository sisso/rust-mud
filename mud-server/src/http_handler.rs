///
/// Handle http request/response into game requests/responses
///
use http_server::{HttpMethod, HttpRequest, HttpRequestId, HttpResponse, HttpStatus};
use mud_domain::controller::{Request, Response};
use mud_domain::errors::{Error, Result};
use mud_domain::game::loader::dto::{ObjData, StaticId};
use mud_domain::game::Game;
use serde_json::{json, Value};

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
    let request_id = http_request.request_id;
    let paths: Vec<&str> = http_request.path.split("/").skip(1).collect();

    let request = match (&http_request.method, paths.as_slice()) {
        (HttpMethod::GET, ["objects"]) => Request::GetObjects,
        (HttpMethod::GET, ["objects", id_str]) => {
            parse_id(request_id, id_str).map(|id| Request::GetObj(id))?
        }
        (HttpMethod::PUT, ["objects", id_str]) => {
            let id = parse_id(request_id, id_str)?;
            let data = parse_body_as_objdata(request_id, http_request.body.as_ref())?;
            assert_request_data_id(request_id, &data, id)?;
            Request::UpdateObj(data)
        }
        (HttpMethod::POST, ["objects"]) => {
            let data = parse_body_as_objdata(request_id, http_request.body.as_ref())?;
            Request::AddObj(data)
        }
        (HttpMethod::GET, ["prefabs"]) => Request::GetPrefabs,
        (HttpMethod::GET, ["prefabs", id_str]) => {
            parse_id(request_id, id_str).map(|id| Request::GetObj(id))?
        }
        (HttpMethod::PUT, ["prefabs", id_str]) => {
            let id = parse_id(request_id, id_str)?;
            let data = parse_body_as_objdata(request_id, http_request.body.as_ref())?;
            assert_request_data_id(request_id, &data, id)?;
            Request::UpdatePrefab(data)
        }
        (HttpMethod::POST, ["prefabs"]) => {
            let data = parse_body_as_objdata(request_id, http_request.body.as_ref())?;
            Request::AddPrefab(data)
        }
        _ => response_not_found(request_id)?,
    };

    Ok(request)
}

fn response_not_found<T>(request_id: HttpRequestId) -> std::result::Result<T, HttpResponse> {
    Err(HttpResponse::new_error(
        request_id,
        HttpStatus::NotFound,
        "not found",
    ))
}

fn response_invalid_id<T>(
    request_id: HttpRequestId,
    id_str: &str,
) -> std::result::Result<T, HttpResponse> {
    let error_msg = &format!("invalid id '{}'", id_str);
    Err(HttpResponse::new_error(
        request_id,
        HttpStatus::Invalid,
        error_msg,
    ))
}

fn parse_body_as_objdata(
    request_id: HttpRequestId,
    body: Option<&Value>,
) -> std::result::Result<ObjData, HttpResponse> {
    match body {
        None => Err(HttpResponse::new_error(
            request_id,
            HttpStatus::Invalid,
            "body require",
        )),
        Some(body) => serde_json::from_value(body.clone()).map_err(|err| {
            HttpResponse::new_error(
                request_id,
                HttpStatus::Invalid,
                &format!("invalid body: {}", err),
            )
        }),
    }
}

fn assert_request_data_id(
    request_id: HttpRequestId,
    data: &ObjData,
    id: u32,
) -> std::result::Result<(), HttpResponse> {
    if data.id.filter(|i| i.as_u32() == id).is_none() {
        response_invalid_id(request_id, &format!("{}", id))
    } else {
        Ok(())
    }
}

fn parse_id<T>(request_id: HttpRequestId, id_str: &str) -> std::result::Result<T, HttpResponse>
where
    T: From<u32>,
{
    match id_str.parse::<u32>() {
        Ok(id) => Ok(id.into()),
        _ => response_invalid_id(request_id, &id_str),
    }
}
