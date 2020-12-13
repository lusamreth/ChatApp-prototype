use super::io::*;
use crate::backend::{PingState, Server};
use crate::domain::*;
use actix::Addr;
use actix_web::{http, web, HttpResponse};

pub async fn register(
    userinfo: web::Json<UserReg>,
    auth_srv: web::Data<Addr<Server>>,
) -> HttpResponse {
    let UserReg { username, password } = userinfo.into_inner();
    let msg_res = auth_srv.send(Registration { username, password }).await;
    match msg_res {
        Ok(res) => match res.cl_id {
            Some(_) => {
                let res = res.to_json();
                let f: RegistrationOutput = serde_json::from_str(&res).expect("cannot parse!");
                HttpResponse::Created().json(&f)
            }
            None => {
                let err = res.map_err();

                let f: ErrResponse<ResponseFeildError> =
                    serde_json::from_str(&err).expect("cannot parse!");

                HttpResponse::Forbidden().json(&f)
            }
        },
        Err(_) => {
            auth_srv.do_send(PingState(5));
            panic!("server hasn't stop!");
        }
    }
}

// give them cookies!
