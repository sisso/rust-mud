use http_server::*;
use serde_json::json;

fn main() {
    let mut server: Box<dyn HttpServer> = HttpServer::new(8333).expect("fail to create");

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(1000));
        let requests = server.take_requests().expect("fail to get requests");
        let responses = requests
            .into_iter()
            .map(|request| handle_request(request))
            .collect();
        server.provide_responses(responses).expect("fail");
    }
}

fn handle_request(request: HttpRequest) -> HttpResponse {
    println!("receive {:?}", request);

    HttpResponse {
        request,
        status: HttpStatus::Ok,
        body: Some(json!({"ok": true})),
    }
}
