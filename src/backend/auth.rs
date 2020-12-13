use super::PingState;
use super::Server;
use crate::domain::*;
use crate::http::io::*;
use actix::*;
use actix_service::{Service, Transform};
use actix_web::{
    dev::{HttpResponseBuilder, ServiceRequest, ServiceResponse},
    http, Error, HttpResponse,
};
use futures::future::{ok, Ready};
use std::future::Future;

pub struct BearerAuth;

pub struct BearerMiddleware<S> {
    service: S,
}

enum BearerFailure {
    InvalidJwt,
    EmptyHeader,
    ExpiredJwt,
}
enum AuthStatus {
    Success,
    Fail(BearerFailure),
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
        let auth_val = req.headers().get("Authorization");
        match auth_val {
            Some(header_val) => {
                match header_val.to_str() {
                    Ok(value) => {
                        let halves = value.split_whitespace().collect::<Vec<&str>>();
                        let d = halves.first();
                        if halves.len() != 2 || halves.first().unwrap().to_lowercase() != "bearer" {
                            BearerFailure::InvalidJwt
                        } else {
                            let token_val = halves.get(1).unwrap();
                            // verify(token);
                            unimplemented!()
                        }
                    }
                    Err(_) => BearerFailure::InvalidJwt,
                }
            }
            None => {}
        };

        let fut = self.service.call(req);
        if true {
            Box::pin(async move {
                let res = fut.await?;
                println!("Hi from response");
                // forward the response
                Ok(res)
            })
        } else {
            Box::pin(async move {
                let mut base = HttpResponseBuilder::new(http::StatusCode::BAD_REQUEST);
                base.body("aplcp");
                Err(Error::from(base))
            })
        }
    }
}

//impl From<String> for actix_web::Error {}
