extern crate actor_ws;
use actix_web::{test::*, web, App, HttpResponse};
use actor_ws::testing_tools::*;
mod logic;
mod registration;
mod room;
mod signal_test;
mod texting;
use actor_ws::http::{io, resources};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct K {
    name: String,
}
async fn backo(body: web::Json<K>) -> HttpResponse {
    return HttpResponse::Ok().json(body.into_inner());
}
#[actix_rt::test]
async fn test_reg() {
    //let srv = build_websocket_mock();
    let backend_srv = backend::Server::create();
    let addrs = backend_srv.start();

    let mut api = init_service(
        App::new()
            .data(addrs.clone())
            .route("/reg", web::post().to(resources::auth_routes::register)),
    )
    .await;

    let json_body = io::UserReg {
        username: "test".to_string(),
        password: "Test2018Pajskdjoak".to_string(),
    };

    let json_req = TestRequest::post()
        .uri("/reg")
        .set_json(&json_body)
        .to_request();
    //let resp_json : io::RegistrationOutput = read_response_json(&mut api, json_req).await;
    let resp_json = call_service(&mut api, json_req).await;

    //println!("final response \n{:#?}",resp);
    println!("final response \n{:#?}", resp_json.response());
}
