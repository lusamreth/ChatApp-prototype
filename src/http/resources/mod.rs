pub use super::io;
pub mod auth_routes;
pub mod websocket;
use actix_files::Files;
use actix_web::web;


pub fn config_server_file(config: &mut web::ServiceConfig) {
    let file_service = Files::new("/static/", "static")
        .index_file("index.html")
        .prefer_utf8(true);
    config.service(file_service);
}

