use serde;
/// implementation neutral structures to deal with http connections
///
use serde_json::json;

mod tiny;

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    DELETE,
    PUT,
}

#[derive(Debug)]
pub enum HttpStatus {
    Ok,
    NotFound,
    Invalid,
    Error,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub request_id: u32,
    pub method: HttpMethod,
    pub path: String,
    pub body: Option<serde_json::Value>,
}

impl HttpRequest {
    pub fn into_error_response(self, status: HttpStatus, error_msg: &str) -> HttpResponse {
        HttpResponse {
            request: self,
            status: status,
            body: Some(json!({ "error": error_msg })),
        }
    }

    pub fn into_success(self) -> HttpResponse {
        HttpResponse {
            request: self,
            status: HttpStatus::Ok,
            body: None,
        }
    }
    pub fn into_success_body<T>(self, value: T) -> HttpResponse
    where
        T: serde::Serialize,
    {
        match serde_json::to_value(value) {
            Ok(mut value) => {
                value.strip_nulls();

                HttpResponse {
                    request: self,
                    status: HttpStatus::Ok,
                    body: Some(value),
                }
            }
            Err(e) => self.into_error_response(HttpStatus::Error, &format!("{}", e)),
        }
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub request: HttpRequest,
    pub status: HttpStatus,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum HttpError {
    Generic(Box<dyn std::error::Error>),
}

pub trait HttpServer {
    fn take_requests(&mut self) -> Result<Vec<HttpRequest>, HttpError>;
    fn provide_responses(&mut self, responses: Vec<HttpResponse>) -> Result<(), HttpError>;
    fn shutdown(&mut self) -> Result<(), HttpError>;
}

impl HttpServer {
    pub fn new(port: u32) -> Result<Box<dyn HttpServer>, HttpError> {
        Ok(Box::new(tiny::TinyHttpServer::new(port)?))
    }
}

trait JsonValueExtra {
    fn strip_nulls(&mut self);
}

impl JsonValueExtra for serde_json::Value {
    fn strip_nulls(&mut self) {
        match self {
            serde_json::Value::Array(array) => array.iter_mut().for_each(|i| {
                i.strip_nulls();
            }),

            serde_json::Value::Object(map) => {
                let mut nulls: Vec<String> = Vec::new();

                for (key, value) in map.iter_mut() {
                    match value {
                        serde_json::Value::Null => nulls.push(key.clone()),
                        _ => value.strip_nulls(),
                    }
                }

                for key in nulls {
                    map.remove(&key);
                }
            }

            _ => {}
        }
    }
}
