use super::*;
use std::io::Cursor;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

pub struct TinyHttpServer {
    server: Server,
    next_request_id: u32,
    pending_requests: Vec<(u32, Request)>,
}

impl TinyHttpServer {
    pub fn new(port: u32) -> Result<Self, HttpError> {
        let server =
            Server::http(format!("0.0.0.0:{}", port)).map_err(|err| HttpError::Generic(err))?;

        Ok(TinyHttpServer {
            server,
            next_request_id: 0,
            pending_requests: vec![],
        })
    }
}

// TODO: do not crash on invalid requests and automatic return errors
impl HttpServer for TinyHttpServer {
    fn take_requests(&mut self) -> Result<Vec<HttpRequest>, HttpError> {
        let mut requests = vec![];

        // collect all requests until there is no more
        loop {
            match self.server.try_recv() {
                Ok(None) => break,
                Ok(Some(mut request)) => {
                    let id = self.next_request_id;
                    self.next_request_id += 1;

                    let mut content_str = String::new();
                    request
                        .as_reader()
                        .read_to_string(&mut content_str)
                        .expect("fail to get request body");

                    let content_json = if content_str.is_empty() {
                        None
                    } else {
                        Some(
                            serde_json::from_str::<serde_json::Value>(&content_str)
                                .expect("fail to parse body"),
                        )
                    };

                    let path = request.url().to_string();

                    let http_request = HttpRequest {
                        request_id: id,
                        method: match request.method() {
                            Method::Post => HttpMethod::POST,
                            Method::Put => HttpMethod::PUT,
                            Method::Delete => HttpMethod::DELETE,
                            Method::Patch => HttpMethod::PATCH,
                            other => HttpMethod::GET,
                        },
                        path: path,
                        body: content_json,
                    };

                    self.pending_requests.push((id, request));
                    requests.push(http_request);
                }
                Err(err) => return Err(HttpError::Generic(Box::new(err))),
            }
        }

        Ok(requests)
    }

    fn provide_responses(&mut self, responses: Vec<HttpResponse>) -> Result<(), HttpError> {
        // this algorithm can be a bit less ugly
        for http_response in responses {
            let index = self
                .pending_requests
                .iter()
                .position(|(id, _)| *id == http_response.request.request_id)
                .expect("could not found request index");

            let (_, request) = self.pending_requests.remove(index);

            let status_code = match http_response.status {
                HttpStatus::Ok => StatusCode::from(200),
                HttpStatus::NotFound => StatusCode::from(404),
                HttpStatus::Invalid => StatusCode::from(403),
                HttpStatus::Error => StatusCode::from(500),
            };

            let headers =
                vec![Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()];

            let body_str = http_response
                .body
                .map(|body_json| serde_json::to_string(&body_json).expect("fail to serialize"))
                .unwrap_or("{}".into());
            let body_len = body_str.len();

            let data = Cursor::new(body_str);

            let response = Response::new(status_code, headers, data, Some(body_len), None);
            request.respond(response);
        }

        if !self.pending_requests.is_empty() {
            panic!("not all request were answered");
        }

        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), HttpError> {
        Ok(())
    }
}
