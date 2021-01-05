use crate::domain::*;
use crate::http::io::*;
use actix_service::{Service, Transform};
// jwt from domian crate
use actix_web::{
    dev::{HttpResponseBuilder, ServiceRequest, ServiceResponse},
    http, Error, HttpRequest, HttpResponse,
};
use futures::future::{ok, Ready};
use jwt::*;
use std::future::Future;

pub struct BearerAuth;

pub struct BearerMiddleware<S> {
    service: S,
}

// S : holder for middleware service
// B : body type response!
impl<S, B> Transform<S> for BearerAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = BearerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(BearerMiddleware { service })
    }
}
use std::pin::Pin;
use std::task::{Context, Poll};

impl<S, B> Service for BearerMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        // looking for bearer key!
        //let matched = req.headers().contains_key("bearer");
        let auth_stat = process_auth_header(&req);
        let fut = self.service.call(req);
        match auth_stat {
            AuthStatus::Success => {
                Box::pin(async move {
                    let res = fut.await?;
                    // forward the response
                    Ok(res)
                })
            }
            // if the access_token expired renew another one!
            AuthStatus::Fail(BearerFailure::ExpiredJwt) => {}
            _ => {
                let resp = get_auth_error(auth_stat);
                let err = ErrResponse::from(resp);
                Box::pin(async move {
                    let base = HttpResponseBuilder::new(http::StatusCode::UNAUTHORIZED)
                        .cookie(http::Cookie::new("name", "apsodksadl"))
                        .json(err);
                    //base.cookie(http::Cookie::new("dom", "0189230zd"));
                    Err(Error::from(base))
                })
            }
        }
    }
}

use std::str::FromStr;
fn cookie_parse(long_cookie_string: &str) -> Result<Vec<http::Cookie>, String> {
    let mut cookie_vec = Vec::new();
    let mut err = None;

    long_cookie_string
        .split(";")
        .for_each(|cookie_str| match http::Cookie::from_str(cookie_str) {
            Ok(cookie) => cookie_vec.push(cookie),
            Err(parsing_err) => {
                err = Some(parsing_err.to_string());
                return;
            }
        });
    // check for throw back
    match err {
        Some(_) => Err(err.unwrap()),
        None => return Ok(cookie_vec),
    }
}

async fn process_refresh_token(req: HttpRequest) {
    let a = req.headers().get("cookie");
    println!("coal");
    //req.head()
}
//impl From<String> for actix_web::Error {}
// how error looks {
/* error: "/auth",
   error-type : "/InvalidJwt",
   Couple struct :
        details:{
            reason : "Bad!"
        }
    Faltten -> error : String ..
   instance:"N/A"
*/

//}
// return auth status but failed only
fn scan_token(cookies: Vec<http::Cookie>) -> Option<String> {
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
fn auth_process(cookies: Vec<http::Cookie>) {}
#[cfg(test)]
mod teser {
    use super::*;
    #[test]
    fn test_scan() {
        let ap = http::Cookie::new("AuthTokenSigniture", "didi");
        let ca = http::Cookie::new("AuthTokenPayload", "69");
        let api = http::Cookie::new("AlsuthTokenSigniture", "322");
        let ca2 = http::Cookie::new("AuthTokenPayload", "69");

        let test_cookies = vec![ap, ca, api, ca2];
        // should only scan the for 2 cookies
        match scan_token(test_cookies) {
            Some(v) => assert_eq!(v, "didi69".to_string()),
            None => panic!(),
        }
    }
}

fn process_auth_header(req: &ServiceRequest) -> AuthStatus {
    // predefined error state!
    let invalid_tk = AuthStatus::Fail(BearerFailure::InvalidToken);
    let parsing_err = AuthStatus::Fail(BearerFailure::ParsingError);

    let mut csrf_captured = String::new();
    let auth_val = req.headers().get("csrf");
    match auth_val {
        Some(header_val) => match header_val.to_str() {
            Ok(value) => csrf_captured.push_str(value),
            Err(_) => return invalid_tk,
        },
        None => return AuthStatus::Fail(BearerFailure::EmptyHeader),
    }

    // authorization process!
    match req.headers().get("cookie") {
        Some(head_val) => match head_val.to_str() {
            Ok(long_cookie_string) => match cookie_parse(long_cookie_string) {
                Ok(cookies) => {
                    //
                    if let Some(token_str) = scan_token(cookies) {
                        let asm =
                            jwt::AccessToken::verify::<utility::CsrfGuard>(token_str.as_str());
                        if let Err(err) = asm {
                            return match err.to_lowercase().as_str() {
                                "expiredsignature" => AuthStatus::Fail(BearerFailure::ExpiredJwt),
                                "invalidtoken" | "invalidsignature" | "invalidalgorithm" => {
                                    AuthStatus::Fail(BearerFailure::BadJwtComponent)
                                }
                                _ => invalid_tk,
                            };
                        }
                        // check the extension
                        let ext = asm.unwrap().claims.extension;
                        if ext.is_none() {
                            return invalid_tk;
                        }
                        if ext.unwrap().token() == csrf_captured {
                            AuthStatus::Success
                        } else {
                            return invalid_tk;
                        }
                    } else {
                        return invalid_tk;
                    }
                    // stop right there
                }
                Err(_) => return parsing_err,
            },
            Err(_) => return parsing_err,
        },
        None => return AuthStatus::Fail(BearerFailure::EmptyCookie),
    }
}

fn scan_refreshtoken(cookies: Vec<http::Cookie>) -> Option<&str> {
    let cookie = None;
    cookies.iter().for_each(|c| {
        if c.name() == "refresh_token" {
            cookie = Some(c.value());
            return;
        }
    });
    return cookie;
}

use actix_web::web;
fn request_newtoken<E>(req: web::HttpRequest, config: web::ServiceConfig, at: TokenClaim<E>) {
    // find user from d
    let mut rtoken = String::new();
    let auth_stat = match req.headers().get("cookie") {
        Some(cookie_str) => match cookie_parse(cookie_str.to_str().unwrap_or("")) {
            Ok(cookies) => match scan_refreshtoken(cookies) {
                Some(tk) => {
                    rtoken.push_str(&tk);
                    AuthStatus::Success
                }
                None => AuthStatus::Fail(BearerFailure::EmptyCookie),
            },
            Err(cookie_err) => AuthStatus::Fail(BearerFailure::ParsingError),
        },
        None => AuthStatus::Fail(BearerFailure::EmptyHeader),
    };

    if rtoken.len() == 0 {
        return;
    }

    let resp = get_auth_error(auth_stat);
    let err = ErrResponse::from(resp);
    at.client_id;
}
