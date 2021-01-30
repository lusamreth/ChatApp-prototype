use super::io::*;
use crate::backend::*;
use crate::domain::*;
use actix::Addr;
use actix_web::{dev::HttpResponseBuilder, http, web, HttpRequest, HttpResponse};
use utils::utility::*;

pub fn insert_auth_tokens(id: uuid::Uuid, re: &mut HttpResponseBuilder)  -> String{

    let ct = utility::CsrfGuard::new();
    let val = ct.token().clone();
    let rt = jwt::RefreshToken::create_refresh_token(id.clone());
    let at = jwt::AccessToken::create_access_token::<utility::CsrfGuard>(
        id,
        DEFAULT_SCOPE.to_vec(),
        Some(ct),
    );
    
    generate_authcookie(at, re);
    val
}

pub async fn register_user(
    req: HttpRequest,
    userinfo: web::Json<UserReg>,
    auth_srv: web::Data<Addr<Server>>,
) -> HttpResponse {

    let UserReg { username, password } = userinfo.into_inner();
    let msg_res = auth_srv.send(Registration { username, password }).await;
    match msg_res {
        Ok(res) => match res.cl_id {
            Some(id) => {

                let mut re = HttpResponseBuilder::new(http::StatusCode::CREATED);
                let res = res.to_json();

                let f: RegistrationOutput = serde_json::from_str(&res).expect("cannot parse!");
                let v = insert_auth_tokens(id, &mut re);

                re.set_header("csrf",v);
                re.json(f)
            }
            None => {
                let err = res.map_err();

                let f: ErrResponse<ResponseFeildError> =
                    serde_json::from_str(&err).expect("cannot parse!");

                HttpResponse::BadRequest().json(&f)
            }
        },
        Err(_) => {
            auth_srv.do_send(PingState(5));
            panic!("server hasn't stop!");
        }
    }
}

// give them cookies!
pub async fn login(
    req: HttpRequest,
    userinfo: web::Json<UserReg>,
    auth_srv: web::Data<Addr<Server>>,
) -> HttpResponse {
    let UserReg { username, password } = userinfo.into_inner();
    let msg_res = auth_srv.send(LoginMessage { username, password }).await;
    match msg_res {
        Ok(res) => match res.cl_id{
            Some(id) => {
                let mut re = HttpResponseBuilder::new(http::StatusCode::ACCEPTED);
                insert_auth_tokens(id.clone(), &mut re);
                re.json(serde_json::json!({
                    "status":"Sucess".to_string(),
                    "id":id.to_string()
                }))
            }
            None => HttpResponse::BadRequest().finish(),
        },
        Err(err) => {
            auth_srv.do_send(PingState(5));
            panic!("server hasn't stop!");
        }
    }
}

// give them cookies!
//
async fn request_newtoken<E>(
    req: web::HttpRequest,
    server: Addr<Server>,
    at: jwt::TokenClaim<E>,
) -> HttpResponse {
    // find user from d

    let map_err = |auth_stat| {
        let a = get_auth_error(auth_stat);
        return HttpResponse::Unauthorized().json(ErrResponse::from(a));
    };

    let uid = at.client_id;
    match req.headers().get("cookie") {
        Some(cookie_str) => match cookie_parse(cookie_str.to_str().unwrap_or("")) {
            Ok(cookies) => match scan_refreshtoken(cookies) {
                Some(tk) => {
                    // cookie has to be valid otherwise it will be rejected
                    match server.send(Rtoken(tk.to_string(), uid)).await {
                        Ok(token_res) => match token_res {
                            Ok(new_token) => {
                                let r = serde_json::json!({
                                    "success":true,
                                    "created_at":utility::timestamp_now()
                                });
                                // split token into three different parts
                                // first and sec are headers and payload
                                // last one is sign
                                let mut resp = HttpResponseBuilder::new(http::StatusCode::OK);
                                resp.json(r);
                                generate_authcookie(new_token, &mut resp);
                                return resp.finish();
                            }
                            Err(e) => map_err(e),
                        },
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    }
                }
                None => map_err(AuthStatus::Fail(BearerFailure::EmptyCookie)),
            },
            Err(cookie_err) => map_err(AuthStatus::Fail(BearerFailure::ParsingError)),
        },
        None => map_err(AuthStatus::Fail(BearerFailure::EmptyHeader)),
    }
}
