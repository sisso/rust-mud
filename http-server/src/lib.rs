/// implementation neutral structures to deal with http connections
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
    pub path: Vec<String>,
    pub body: Option<serde_json::Value>,
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
