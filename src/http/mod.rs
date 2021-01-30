pub mod io;
pub mod resources;
use actix::*;
<<<<<<< HEAD
=======
<<<<<<< HEAD
>>>>>>> d41459f (Improving authentication logic!)
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};
use env_logger::Env;
//use resources::config_server_file;
const SERVER_ADDRESS: &'static str = "127.0.0.1:8030";
use crate::{backend, domain, pipe};

use self::resources::config_server_file;

async fn test_url(req: HttpRequest) -> HttpResponse {
    //dbg!("{:#?}", &req.match_info().get("socket"));

<<<<<<< HEAD
=======
=======
use actix_web::{http, middleware::*, web, App, HttpRequest, HttpResponse, HttpServer};
use env_logger::Env;
//use resources::config_server_file;
const SERVER_ADDRESS: &'static str = "127.0.0.1:8030";
use self::resources::*;
use crate::{backend, domain, pipe};
use actix_cors::Cors;

async fn test_url(req: HttpRequest) -> HttpResponse {
    //dbg!("{:#?}", &req.match_info().get("socket"));
>>>>>>> 21fb43b (Handshake authentication)
>>>>>>> d41459f (Improving authentication logic!)
    HttpResponse::Ok().body("suck")
}

#[actix_web::main]
pub async fn build() -> std::io::Result<()> {
    let server = backend::Server::create().start();
    //env_logger::init();
    dbg!("ok");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    HttpServer::new(move || {
<<<<<<< HEAD
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
=======
<<<<<<< HEAD
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
=======
        let cors = Cors::default()
            .allowed_origin("http://localhost:8030")
            .allowed_methods(vec!["GET", "POST"])
            .supports_credentials()
            .allow_any_header()
            // .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .max_age(3600)
            .allowed_header(http::header::CONTENT_TYPE);

        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(cors)
            .wrap(Compress::new(http::ContentEncoding::Gzip))
>>>>>>> 21fb43b (Handshake authentication)
>>>>>>> d41459f (Improving authentication logic!)
            .data(server.clone())
            .default_service(web::route().to(|| HttpResponse::Ok()))
            //.route("/", web::get().to(resources::registration::retreive_user))
            .configure(config_server_file)
<<<<<<< HEAD
=======
<<<<<<< HEAD
>>>>>>> d41459f (Improving authentication logic!)
            .wrap(backend::auth::BearerAuth)
            .route("/regu", web::to(resources::auth_routes::register))
            .route("/ws/{socket_id}", web::to(resources::websocket::chat_route))
            .route("/test/{socket}", web::to(test_url))
<<<<<<< HEAD
=======
=======
            .service(
                web::scope("/api/v1")
                    .service(web::resource("/register").to(auth_routes::register_user))
                    .service(web::resource("/login").to(auth_routes::login)),
            )
            .service(
                web::scope("/ws")
                    .wrap(backend::auth::BearerAuth)
                    .service(web::resource("/{socket_id}").to(websocket::chat_route)),
            )
        //.wrap(backend::auth::BearerAuth)
        // below is only for testing
        // when running test please uncomment this route
        //.route("/ws/{socket_id}", web::to(resources::websocket::chat_route))
        //.route("/test/{socket}", web::to(test_url))
>>>>>>> 21fb43b (Handshake authentication)
>>>>>>> d41459f (Improving authentication logic!)
    })
    .bind(SERVER_ADDRESS)?
    .run()
    .await
}
