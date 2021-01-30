pub mod utility {
    use super::*;
    pub use crate::domain::{utility, *};
    use actix_web::{dev::HttpResponseBuilder, http, web};
    use std::str::FromStr;
    use time::{Duration, OffsetDateTime};

    pub fn generate_authcookie<'a>(new_token: String, resp: &mut HttpResponseBuilder) {
        println!("fresh token {:#?}",new_token); 
        let mut i = 0;

        let mut dot_idx = new_token.chars().enumerate().fold([0;2],|mut acc,e| {
            if e.1 == '.' {
                acc[i] = e.0;
                i += 1;
            }
            acc
        });

        let p1 = new_token.get(0..dot_idx[1]).unwrap();
        
        let sign = new_token.get(dot_idx[1] + 1..).unwrap();

        let exp = |time| {
            let mut oft = OffsetDateTime::now_utc();
            oft += time;
            return oft;
        };

        let payload_cookie = http::Cookie::build("AuthTokenPayload", p1)
            .path("/")
            .expires(exp(Duration::minutes(15)))
            .secure(false)
            .finish();

        let sig_cookie = http::Cookie::build("AuthTokenSigniture", sign)
            .http_only(true)
            .path("/")
            .secure(false)
            .expires(exp(Duration::days(7)))
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
        let buffer = cookies.into_iter().fold(vec![String::new();2],|mut acc,tkstr| {
            match tkstr.name() {
                "AuthTokenPayload" if state.0 != 1 => {
                    state.0 += 1;
                    acc[0].push_str(tkstr.value());
                    acc
                }
                "AuthTokenSigniture" if state.1 != 1 => {
                    acc[1].push_str(tkstr.value());
                    state.1 += 1;
                    acc
                }
                _ => acc
            }
        });
        // one of token missin!
        let mut after = 0 ;
        let p = buffer.into_iter().fold(String::new(),|mut acc,b|{
            println!("this is b {}",b);
            if after == 1 {
                acc.push('.');
            }
            acc.push_str(b.as_str());

            after = 1;
            acc
        });

        println!("faina {:#?}",p);
        return Some(p);
    }
}

pub mod authprocessor {

    use super::utility::*;
    use actix_service::{Service, Transform};
    use actix_web::http;

    fn process_auth_header(reqhead: &http::HeaderMap,f:impl FnOnce(&http::HeaderMap) -> Option<String>) -> AuthStatus {
        
        // predefined error state!
        let invalid_tk = AuthStatus::Fail(BearerFailure::InvalidToken);
        let parsing_err = AuthStatus::Fail(BearerFailure::ParsingError);
        let empty_header = AuthStatus::Fail(BearerFailure::EmptyHeader);

        let mut csrf_captured = String::new();
        match f(&reqhead){
            Some(header_val) => csrf_captured.push_str(header_val.as_str()),
            None => return empty_header,
        };
        let head_keys = reqhead.iter().find(|(k,_)| k.to_string().to_lowercase() == "cookie".to_string());
        // authorization process!
        match head_keys {
            Some(head_val) => match head_val.1.to_str() {
                Ok(long_cookie_string) => match cookie_parse(long_cookie_string) {
                    Ok(cookies) => {
                        //
                        println!("le cookies {:#?}",cookies);
                        if let Some(token_str) = scan_token(cookies) {
                            let asm =
                                jwt::AccessToken::verify::<utility::CsrfGuard>(token_str.as_str());
                            if let Err(err) = asm {
                                return match err.to_lowercase().as_str() {
                                    "expiredsignature" => {
                                        AuthStatus::Fail(BearerFailure::ExpiredJwt)
                                    }
                                    "invalidtoken" | "invalidsignature" | "invalidalgorithm" => {
                                        AuthStatus::Fail(BearerFailure::BadJwtComponent)
                                    }
                                    _ => invalid_tk,
                                };
                            }
                            // check the extension
                            let ext = asm.unwrap().claims.extension;
                            let t = ext.unwrap().token();

                            println!("this ex {:#?}",t);
                            println!("ex {:#?}",csrf_captured);
                            if  t== csrf_captured {
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
    
    pub fn authorize_http_req(reqhead:&http::HeaderMap) -> AuthStatus{
       let get_csrf = |head:&http::HeaderMap| head.get("csrf").map(|val| {
            val.to_str().unwrap_or("").to_string()
       });
       process_auth_header(reqhead,get_csrf)
    }

    pub fn authorize_wshandshake(reqhead:&http::HeaderMap) -> AuthStatus{

        let fetch_csrf = |reqhead: &http::HeaderMap| {
           reqhead.get("sec-websocket-protocol").map(|prot| {
                let hs = prot.to_str().unwrap_or("");
                let from_url = |s:&str| s.replace("-","+");
                let mut c = hs.split(",").fold(String::new(),|mut acc,e|{
                    if e != "token"{
                        acc.push_str(from_url(e.trim()).as_str());
                        acc
                    }else{
                        acc
                    }
                });
                c.push_str("=");
                println!("c {:#?}",c);
                return c;
                //c.map(|val| val.split_whitespace().skip(1).next().unwrap_or("").to_string());
                //c.unwrap().split_whitespace().skip(1).next().unwrap_or("").to_string()
           })
        };

       process_auth_header(reqhead,fetch_csrf)
    }
}
