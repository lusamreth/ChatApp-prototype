pub mod backend;
pub mod domain;
pub mod http;
pub mod pipe;

pub mod testing_tools {
    pub use super::*;
    pub use actix::*;
    pub use actix_web::{test, web, App};
    pub use futures::*;

    pub fn build_fake_client(num: i32) -> Vec<domain::Client> {
        let len = num as usize;
        let mut res = Vec::with_capacity(len);
        for _ in 0..num {
            let mock_usr = domain::User::new("super diteched".to_string(), "192739".to_string());
            res.push(domain::Client::new(mock_usr));
        }
        return res;
    }

    // default clients to 100;
    pub fn build_websocket_mock() -> test::TestServer {
        let mock_server = backend::Server::create();
        let mockclients = build_fake_client(0);
        mockclients.into_iter().for_each(|client| {
            let mut client_handle = mock_server.clients.write().unwrap();
            client_handle.insert(client.client_id.clone(), client);
        });
        let addrs = mock_server.start();
        let srv = test::start(move || {
            App::new()
                .data(addrs.clone())
                .route("/", web::to(http::resources::websocket::chat_route))
<<<<<<< HEAD
                .route("/reg", web::to(http::resources::auth_routes::register))
=======
<<<<<<< HEAD
                .route("/reg", web::to(http::resources::auth_routes::register))
=======
                .route("/reg", web::to(http::resources::auth_routes::register_user))
>>>>>>> 21fb43b (Handshake authentication)
>>>>>>> d41459f (Improving authentication logic!)
        });
        return srv;
    }
}
