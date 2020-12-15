use super::utility;
use jsonwebtoken::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub type JwtRes<T> = Result<TokenData<TokenClaim<T>>, String>;
// store user_id,username,date
//
#[derive(Serialize, Deserialize)]
pub struct TokenClaim<T> {
    client_id: String,
    aud: String,
    sub: String,
    iss: String,
    exp: usize,
    iat: usize,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extension: Option<T>,
}

// use to invalidate the access_token
#[derive(Serialize, Deserialize)]
pub struct TokenVersion {
    #[serde(default)]
    token_version: usize,
}
impl<T> TokenClaim<T> {
    fn create_scope(input: Vec<&str>) -> String {
        let mut scope = String::new();
        let wl = input.len();
        input.iter().enumerate().for_each(|(i, perm)| {
            let mut padding = "\u{A0}";
            if i == 0 || i == wl - 1 {
                padding = "";
            }
            scope.push_str(&format!("{}{}", perm, padding));
        });
        return scope;
    }

    // create token claim for access_token
    fn access_token(client_id: String, input_scope: Vec<&str>) -> Self {
        let iat = utility::timestamp_now().as_secs() as usize;
        let scope = Self::create_scope(input_scope);
        TokenClaim {
            client_id,
            scope,
            iss: "/ChatServer".to_string(),
            aud: "/static/chat-client".to_string(),
            iat,
            // 15 mins expiration
            exp: 60 * 15,
            //sub field- not sure about this one!!
            sub: "randomized!!".to_string(),
            extension: None,
        }
    }

    // create token claim for refresh_token
    fn refresh_token(client_id: String) -> Self {
        let iat = utility::timestamp_now().as_secs() as usize;
        let scope = Self::create_scope(vec!["Create-Access-token", "Renewal", "Session"]);
        TokenClaim {
            client_id,
            scope,
            iss: "/ChatServer".to_string(),
            aud: "/static/chat-client".to_string(),
            iat,
            // 7 days expiration
            exp: 60 * 15,
            //sub field- not sure about this one!!
            sub: "randomized!!".to_string(),
            extension: None,
        }
    }

    fn add_extension(&mut self, ext: T) {
        self.extension = Some(ext);
    }
}

lazy_static! {
    // private keys
    static ref AKEY : &'static [u8] = include_bytes!("../../access_token/private_key.pem");
    static ref RKEY : &'static [u8] = include_bytes!("../../access_token/private_key.pem");
    // public keys
    static ref PUB_AKEY : &'static [u8] = include_bytes!("../../access_token/public_key.pem");
    static ref PUB_RKEY : &'static [u8] = include_bytes!("../../access_token/public_key.pem");
}
//const Ajey: String = encodingkey::from_rsa_pem(read_key).expect("failed to encodekey");

use uuid::Uuid;
// 15 mins = 15 * 60 = 900s
pub struct AccessToken;
impl AccessToken {
    pub fn create_access_token(clid: Uuid, scope: Vec<&str>) -> String {
        let mut header = Header::new(Algorithm::HS384);
        let id = Uuid::to_string(&clid);
        let newtoken = TokenClaim::<String>::access_token(id, scope);
        //let read_key = include_bytes!("../../access_token/private_key.pem");
        jsonwebtoken::EncodingKey::from_rsa_pem(&AKEY).expect("failed to encodekey");
        // creating key
        let read_key = include_bytes!("../../access_token/private_key.pem");
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(read_key).expect("failed to encodekey");

        encode(&header, &newtoken, &key).expect("Cannot encode jwt")
    }

    pub fn verify(token: &str) -> JwtRes<String> {
        let validation = Validation::new(Algorithm::HS384);
        let dk = DecodingKey::from_rsa_pem(&PUB_AKEY).expect("Cannot unwrap token's key");
        // String extension is just placeholder!!
        match jsonwebtoken::decode::<TokenClaim<String>>(token, &dk, &validation) {
            Ok(token) => Ok(token),
            Err(token_err) => Err(token_err.to_string()),
        }
    }
}
pub struct RefreshToken;

impl RefreshToken {
    pub fn create_refresh_token(clid: Uuid) -> String {
        let mut header = Header::new(Algorithm::HS384);
        //let id = Uuid::to_string(&clid);
        let mut newtoken = TokenClaim::<TokenVersion>::refresh_token(clid.to_string());

        newtoken.add_extension(TokenVersion { token_version: 0 });
        // creating key
        let read_key = include_bytes!("../../refresh_token/private_key.pem");
        let key = EncodingKey::from_rsa_pem(read_key).expect("Failed to encodekey");
        encode(&header, &newtoken, &key).expect("Cannot encode jwt")
    }

    pub fn verify(token: &str, tkv: usize) -> JwtRes<TokenVersion> {
        let validation = Validation::new(Algorithm::HS384);
        let dk = DecodingKey::from_rsa_pem(&PUB_RKEY).expect("Cannot unwrap token's key");
        // String extension is just placeholder!!
        let token_res = jsonwebtoken::decode::<TokenClaim<TokenVersion>>(token, &dk, &validation);
        match token_res {
            Ok(token) => Ok(token),
            Err(token_error) => Err(token_error.to_string()),
        }
    }
}
