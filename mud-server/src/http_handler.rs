///
/// Handle http request/response into game requests/responses
///
use http_server::{HttpMethod, HttpRequest, HttpRequestId, HttpResponse, HttpStatus};
use logs::*;
use mud_domain::controller;
use mud_domain::errors::{Error, Result};
use mud_domain::game::container::Container;
use mud_domain::game::loader::dto::{ObjData, StaticId};
use mud_domain::game::Game;
use serde_json::{json, Value};

type HttpResult = std::result::Result<HttpResponse, HttpResponse>;

pub fn handle_requests(game: &mut Game, requests: Vec<HttpRequest>) -> Vec<HttpResponse> {
    requests
        .into_iter()
        .map(|http_request| handle_request(game, &http_request))
        .map(&fold_result)
        .collect()
}

fn handle_request(game: &mut Game, http_request: &HttpRequest) -> HttpResult {
    let request_id = http_request.request_id;
    let paths: Vec<&str> = http_request.path.split("/").skip(1).collect();

    match (&http_request.method, paths.as_slice()) {
        (HttpMethod::GET, ["objects"]) => handle_get_objects(&game, request_id),
        (HttpMethod::GET, ["objects", id_str]) => {
            handle_get_object_by_id(&game, request_id, id_str)
        }
        (HttpMethod::GET, ["prefabs"]) => handle_get_prefabs(&game, request_id),
        (HttpMethod::GET, ["prefabs", id_str]) => handle_get_prefab_by_id(game, request_id, id_str),
        (HttpMethod::PUT, ["prefabs", id_str]) => {
            handle_put_prefab(game, request_id, id_str, http_request.body.as_ref())
        }
        (HttpMethod::POST, ["prefabs"]) => {
            handle_post_prefab(game, request_id, http_request.body.as_ref())
        }
        _ => response_not_found(request_id),
    }
}

fn handle_get_prefab_by_id(game: &Game, request_id: u32, id_str: &str) -> HttpResult {
    let id = parse_id(request_id, id_str)?;
    let data = controller::handle_request_get_prefab(&game.container, id)
        .map_err(|err| handle_error::<ObjData>(request_id, err))?;
    Ok(HttpResponse::new_success_body(request_id, data))
}

fn handle_get_prefabs(game: &Game, request_id: u32) -> HttpResult {
    let prefabs = controller::handle_request_get_prefabs(&game.container)
        .map_err(|err| handle_error::<Vec<ObjData>>(request_id, err))?;
    Ok(HttpResponse::new_success_body(
        request_id,
        json!({ "prefabs": prefabs }),
    ))
}
fn handle_get_object_by_id(game: &Game, request_id: u32, id_str: &str) -> HttpResult {
    let id = parse_id(request_id, id_str)?;
    let data = controller::handle_request_get_object(&game.container, id)
        .map_err(|err| handle_error::<ObjData>(request_id, err))?;
    Ok(HttpResponse::new_success_body(request_id, data))
}

fn handle_get_objects(game: &Game, request_id: u32) -> HttpResult {
    let objects = controller::handle_request_get_objects(&game.container)
        .map_err(|err| handle_error::<Vec<ObjData>>(request_id, err))?;
    Ok(HttpResponse::new_success_body(
        request_id,
        json!({ "objects": objects }),
    ))
}

fn handle_post_prefab(game: &mut Game, request_id: u32, body: Option<&Value>) -> HttpResult {
    let data = parse_body_as_objdata(request_id, body)?;
    let static_id = controller::handle_request_add_prefab(&mut game.container, data)
        .map_err(|err| handle_error::<StaticId>(request_id, err))?;
    Ok(HttpResponse::new_success_body(
        request_id,
        json!({"static_id": static_id.as_u32()}),
    ))
}

fn handle_put_prefab(
    game: &mut Game,
    request_id: u32,
    id_str: &str,
    body: Option<&Value>,
) -> HttpResult {
    let id = parse_id(request_id, id_str)?;
    let data = parse_body_as_objdata(request_id, body)?;
    let _ = assert_request_data_id(request_id, &data, id)?;
    controller::handle_request_update_prefab(&mut game.container, data)
        .map_err(|err| handle_error::<()>(request_id, err))?;
    Ok(HttpResponse::new_success(request_id))
}

fn handle_error<T>(request_id: HttpRequestId, err: Error) -> HttpResponse {
    match err {
        Error::NotFoundStaticId(_) | Error::NotFoundFailure => HttpResponse::new_error(
            request_id,
            HttpStatus::NotFound,
            &format!("{}", "not found"),
        ),
        err => HttpResponse::new_error(request_id, HttpStatus::Error, &format!("{}", err)),
    }
}

fn response_not_found(request_id: HttpRequestId) -> HttpResult {
    Err(HttpResponse::new_error(
        request_id,
        HttpStatus::NotFound,
        "not found",
    ))
}

fn response_invalid_id(request_id: HttpRequestId, id_str: &str) -> HttpResponse {
    let error_msg = &format!("invalid id '{}'", id_str);
    HttpResponse::new_error(request_id, HttpStatus::Invalid, error_msg)
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
        Err(response_invalid_id(request_id, &format!("{}", id)))
    } else {
        Ok(())
    }
}

fn fold_result<T>(result: std::result::Result<T, T>) -> T {
    match result {
        Ok(v) => v,
        Err(v) => v,
    }
}

fn parse_id<T>(request_id: HttpRequestId, id_str: &str) -> std::result::Result<T, HttpResponse>
where
    T: From<u32>,
{
    match id_str.parse::<u32>() {
        Ok(id) => Ok(id.into()),
        _ => Err(response_invalid_id(request_id, &id_str)),
    }
}
