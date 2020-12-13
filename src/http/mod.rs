pub mod io;
pub mod resources;
use actix::*;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};
use env_logger::Env;
//use resources::config_server_file;
const SERVER_ADDRESS: &'static str = "127.0.0.1:8030";
use crate::{backend, domain, pipe};

use self::resources::config_server_file;

async fn test_url(req: HttpRequest) -> HttpResponse {
    //dbg!("{:#?}", &req.match_info().get("socket"));

    HttpResponse::Ok().body("suck")
}

#[actix_web::main]
pub async fn build() -> std::io::Result<()> {
    let server = backend::Server::create().start();
    //env_logger::init();
    dbg!("ok");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(server.clone())
            .default_service(web::route().to(|| HttpResponse::Ok()))
            //.route("/", web::get().to(resources::registration::retreive_user))
            .configure(config_server_file)
            .wrap(backend::auth::BearerAuth)
            .route("/regu", web::to(resources::auth_routes::register))
            .route("/ws/{socket_id}", web::to(resources::websocket::chat_route))
            .route("/test/{socket}", web::to(test_url))
    })
    .bind(SERVER_ADDRESS)?
    .run()
    .await
}
