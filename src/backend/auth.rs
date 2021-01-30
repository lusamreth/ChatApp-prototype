use crate::domain::*;
use crate::http::io::*;
use actix_service::{Service, Transform};
// jwt from domian crate
use super::utils::{authprocessor, utility::*};
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

        //let auth_stat = authprocessor::process_auth_header(&req.headers());
        let auth_stat = match req.headers().get("Sec-Websocket-Protocol") {
            Some(_) => authprocessor::authorize_wshandshake(&req.headers()),
            None => authprocessor::authorize_http_req(&req.headers())
        };

        let fut = self.service.call(req);
        println!(":{:#?}", auth_stat);
        match auth_stat {
            AuthStatus::Success => {
                Box::pin(async move {
                    let res = fut.await?;
                    // forward the response
                    Ok(res)
                })
            }
            // if the access_token expired renew another one!
            //AuthStatus::Fail(BearerFailure::ExpiredJwt) => {}
            _ => {
                let resp = get_auth_error(auth_stat);
                let err = ErrResponse::from(resp);
                Box::pin(async {
                    let base = HttpResponseBuilder::new(http::StatusCode::UNAUTHORIZED).json(err);
                    Err(Error::from(base))
                })
            }
        }
    }
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
