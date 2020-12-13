use super::utility;
use jsonwebtoken::*;
use serde::{Deserialize, Serialize};
// store user_id,username,date
//
#[derive(Serialize, Deserialize)]
struct TokenClaim {
    client_id: String,
    aud: String,
    sub: String,
    iss: String,
    exp: usize,
    iat: usize,
    scope: String,
}

impl TokenClaim {
    fn access_token(client_id: String, input_scope: Vec<&str>) -> Self {
        let mut scope = String::new();
        let wl = input_scope.len();
        input_scope.iter().enumerate().for_each(|(i, perm)| {
            let mut padding = "\u{A0}";
            if i == 0 || i == wl - 1 {
                padding = "";
            }
            scope.push_str(&format!("{}{}", perm, padding));
        });
        let iat = utility::timestamp_now().as_secs() as usize;
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
        }
    }

    fn refresh_token() {}
}
// 15 mins = 15 * 60 = 900s
use uuid::Uuid;
pub fn create_access_token(clid: Uuid, scope: Vec<&str>) -> String {
    let mut header = Header::new(Algorithm::HS384);
    let id = Uuid::to_string(&clid);
    let newtoken = TokenClaim::access_token(id, scope);
    let read_key = include_bytes!("../../access_token/private_key.pem");
    let key = EncodingKey::from_rsa_pem(read_key).expect("Failed to encodekey");
    encode(&header, &newtoken, &key).expect("Cannot encode jwt")
    //header.
    //encode()
}

pub fn create_refresh_token() {
    let mut header = Header::new(Algorithm::HS384);
    //let id = Uuid::to_string(&clid);
    let newtoken = TokenClaim::refresh_token();
    let read_key = include_bytes!("../../private_key.pem");
    let key = EncodingKey::from_rsa_pem(read_key).expect("Failed to encodekey");
    encode(&header, &newtoken, &key).expect("Cannot encode jwt")
}
