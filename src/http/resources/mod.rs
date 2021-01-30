pub use super::io;
pub mod auth_routes;
pub mod websocket;
use actix_files::Files;
<<<<<<< HEAD
=======
<<<<<<< HEAD
>>>>>>> d41459f (Improving authentication logic!)
use actix_web::web;

pub fn config_server_file(config: &mut web::ServiceConfig) {
    let file_service = Files::new("/static/", "static/")
<<<<<<< HEAD
=======
=======
use actix_web::{self, web};

pub fn config_server_file(config: &mut web::ServiceConfig) {
    let file_service = Files::new("/static/", "static")
>>>>>>> 21fb43b (Handshake authentication)
>>>>>>> d41459f (Improving authentication logic!)
        .index_file("index.html")
        .prefer_utf8(true);
    config.service(file_service);
}

// pub fn config_registration(data:web::Data<backend::Server>,config: &mut web::ServiceConfig){
//     config.app_data(data).route("/", web::post().to(registration::register_user));
// }
<<<<<<< HEAD
=======
<<<<<<< HEAD
=======
//
pub mod utility {
    use super::*;
    pub use crate::domain::{utility, *};
    use actix_web::{dev::HttpResponseBuilder, http, web};
    use std::str::FromStr;
    use time::{Duration, OffsetDateTime};

    pub fn generate_authcookie<'a>(new_token: String, resp: &mut HttpResponseBuilder) {
        let sp = new_token.split(".").collect::<Vec<&str>>();
        let p1 = sp.get(0..2).unwrap();

        let p1 = p1.iter().fold(String::new(), |mut acc, e| {
            acc.push_str(e);
            return acc;
        });
        let sign = sp.get(2).unwrap();
        let mut oft = OffsetDateTime::now_utc();
        oft += Duration::minutes(15);

        let mut payload_cookie = http::Cookie::new("AuthTokenPayload", p1.clone());
        payload_cookie.set_expires(oft);
        payload_cookie.set_secure(false);

        let mut sig_cookie = http::Cookie::build("AuthTokenSigniture", *sign)
            .secure(true)
            .expires(oft)
            .finish();
        resp.cookie(payload_cookie).cookie(sig_cookie);
    }

    pub fn scan_refreshtoken<'a>(cookies: Vec<http::Cookie>) -> Option<String> {
        let mut cookie = None;
        cookies.into_iter().for_each(|c| {
            if c.name() == "refresh_token" {
                cookie = Some(c.value().to_string());
                return;
            }
        });
        return cookie;
    }

    pub fn cookie_parse(long_cookie_string: &str) -> Result<Vec<http::Cookie>, String> {
        let mut cookie_vec = Vec::new();
        let mut err = None;

        long_cookie_string.split(";").for_each(|cookie_str| {
            match http::Cookie::from_str(cookie_str) {
                Ok(cookie) => cookie_vec.push(cookie),
                Err(parsing_err) => {
                    err = Some(parsing_err.to_string());
                    return;
                }
            }
        });
        // check for throw back
        match err {
            Some(_) => Err(err.unwrap()),
            None => return Ok(cookie_vec),
        }
    }

    pub fn scan_token(cookies: Vec<http::Cookie>) -> Option<String> {
        let mut state = (0, 0);
        //do this to prevent scanning same token multiple times!
        let auth_tokens = cookies.iter().filter(|cookie| match cookie.name() {
            "AuthTokenPayload" if state.0 != 1 => {
                state.0 += 1;
                true
            }
            "AuthTokenSigniture" if state.1 != 1 => {
                state.1 += 1;
                true
            }
            _ => false,
        });
        // one of token missin!
        let mut p = String::new();
        let mut t_len = 0;
        auth_tokens.take(2).for_each(|partial| {
            t_len += 1;
            p.push_str(partial.value())
        });
        if t_len != 2 {
            return None;
        }
        print!("p : {:#?}", t_len);
        return Some(p);
    }
}
>>>>>>> 21fb43b (Handshake authentication)
>>>>>>> d41459f (Improving authentication logic!)
