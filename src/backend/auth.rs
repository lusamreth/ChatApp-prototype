use crate::domain::*;
use crate::http::io::*;
use actix_service::{Service, Transform};
use actix_web::{
    dev::{HttpResponseBuilder, ServiceRequest, ServiceResponse},
    http, Error, HttpRequest, HttpResponse,
};
use futures::future::{ok, Ready};
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
        println!("asdasld");
        let fut = self.service.call(req);
        if let AuthStatus::Success = auth_stat {
            //if true {
            Box::pin(async move {
                let res = fut.await?;
                // forward the response
                Ok(res)
            })
        } else {
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
use std::str::FromStr;
fn process_auth_header(req: &ServiceRequest) -> AuthStatus {
    match req.headers().get("cookie") {
        Some(head_val) => match head_val.to_str() {
            Ok(long_cookie_string) => match cookie_parse(long_cookie_string) {
                Ok(cookies) => {
                    scan_token(cookies);
                }
                Err(_) => {}
            },
            Err(_) => {}
        },
        None => {} //None => AuthStatus::Fail(BearerFailure::EmptyCookie),
    };
    let auth_val = req.headers().get("Authorization");
    match auth_val {
        Some(header_val) => {
            match header_val.to_str() {
                Ok(value) => {
                    let halves = value.split_whitespace().collect::<Vec<&str>>();
                    println!("P{:#?}", halves);
                    if halves.len() != 2 || halves.first().unwrap().to_lowercase() != "bearer" {
                        AuthStatus::Fail(BearerFailure::InvalidToken)
                    } else {
                        let token_val = halves.get(1).unwrap();
                        if jwt::AccessToken::verify(token_val).is_err() {
                            AuthStatus::Fail(BearerFailure::InvalidToken)
                        } else {
                            AuthStatus::Success
                        }
                        // verify(token);
                    }
                }
                Err(_) => AuthStatus::Fail(BearerFailure::InvalidToken),
            }
        }
        None => AuthStatus::Fail(BearerFailure::EmptyHeader),
    }
}

fn cookie_fetch(req_header: &http::HeaderMap) {}

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

fn scan_token(cookies: Vec<http::Cookie>) {
    let mut state = (0,0);
    let mut auth_tokens = cookies.iter().filter(|cookie|{
        match cookie.name() {
            "AuthTokenPayload" if state.0 != 1 => {
                state.0 += 1;
                true
            },
            "AuthTokenSigniture" if state.1 !=1 => {
                state.1 += 1;
                true
            },
            _ => false
        }
    });
    let mut p = String::new();
    let mut scan = || {
        let mut i = 0;
        loop {
            match auth_tokens.next() {
                Some(token) => {
                    println!("i {}",i);
                    if i >= 2 {
                        break;
                    }
                    p.push_str(token.value());
                    i += 1;
                }
                None => break,
            };
        }
    };
    scan();
    println!("token {}", p);
}

#[test]
fn test_scan() {
    let ap = http::Cookie::new("AuthTokenSigniture", "didi");
    let ca  = http::Cookie::new("AuthTokenPayload", "69");
    let api = http::Cookie::new("AuthTokenSigniture", "322");
    let test_cookies = vec![ap,api,ca];
    scan_token(test_cookies);
}
