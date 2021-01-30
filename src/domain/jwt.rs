use super::utility;
use jsonwebtoken::*;
use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type JwtRes<T> = Result<TokenData<TokenClaim<T>>, String>;
// store user_id,username,date
//
#[derive(Serialize, Deserialize)]
pub struct TokenClaim<T> {
    pub client_id: String,
    aud: String,
    sub: String,
    iss: String,
    exp: usize,
    iat: usize,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<T>,
}

// use to invalidate the access_token
#[derive(Serialize, Deserialize)]
pub struct TokenVersion {
    #[serde(default)]
    pub token_version: usize,
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
        let exp = utility::timestamp_now().as_secs() + 60 * 15;

        TokenClaim {
            client_id,
            scope,
            iss: "/ChatServer".to_string(),
            aud: "/static/chat-client".to_string(),
            iat,
            // 15 mins expiration
            exp: exp as usize,
            //sub field- not sure about this one!!
            sub: "randomized!!".to_string(),
            extension: None,
        }
    }

    // create token claim for refresh_token
    fn refresh_token(client_id: String) -> Self {
        let iat = utility::timestamp_now().as_secs() as usize;
        let scope = Self::create_scope(vec!["Create-Access-token", "Renewal", "Session"]);
        let exp = utility::timestamp_now().as_secs() + 3600 * 24 * 7;
        TokenClaim {
            client_id,
            scope,
            iss: "/ChatServer".to_string(),
            aud: "/static/chat-client".to_string(),
            iat,
            // 7 days expiration
            exp: exp as usize,
            //sub field- not sure about this one!!
            sub: "randomized!!".to_string(),
            extension: None,
        }
    }

    pub fn add_extension(&mut self, ext: T) {
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
    pub fn create_access_token<Ext>(clid: Uuid, scope: Vec<&str>, ext: Option<Ext>) -> String
    where
        Ext: DeserializeOwned + Serialize,
    {
        let header = Header::new(Algorithm::RS384);

        let id = Uuid::to_string(&clid);

        let mut newtoken = TokenClaim::<Ext>::access_token(id, scope);
        if ext.is_some() {
            newtoken.add_extension(ext.unwrap());
        }
        jsonwebtoken::EncodingKey::from_rsa_pem(&AKEY).expect("failed to encodekey");

        // creating key
        let read_key = include_bytes!("../../access_token/private_key.pem");
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(read_key).expect("failed to encodekey");

        encode(&header, &newtoken, &key).expect("Cannot encode jwt")
    }

    pub fn verify<Ext>(token: &str) -> JwtRes<Ext>
    where
        Ext: DeserializeOwned + Serialize,
    {
        let validation = Validation::new(Algorithm::RS384);
        println!("v :{:#?} ", validation.validate_exp);
        let dk = DecodingKey::from_rsa_pem(&PUB_AKEY).expect("Cannot unwrap token's key");
        // String extension is just placeholder!!

        match jsonwebtoken::decode::<TokenClaim<Ext>>(token, &dk, &validation) {
            Ok(token) => Ok(token),
            Err(token_err) => Err(token_err.to_string()),
        }
    }
}
pub struct RefreshToken;

impl RefreshToken {
    pub fn create_refresh_token(clid: Uuid) -> String {
        let header = Header::new(Algorithm::RS384);
        //let id = Uuid::to_string(&clid);
        let mut newtoken = TokenClaim::<TokenVersion>::refresh_token(clid.to_string());

        newtoken.add_extension(TokenVersion { token_version: 0 });
        // creating key
        let read_key = include_bytes!("../../refresh_token/private_key.pem");
        let key = EncodingKey::from_rsa_pem(read_key).expect("Failed to encodekey");

        encode(&header, &newtoken, &key).expect("Cannot encode jwt")
    }

    pub fn verify(token: &str) -> JwtRes<TokenVersion> {
        let validation = Validation::new(Algorithm::RS384);
        let dk = DecodingKey::from_rsa_pem(&PUB_RKEY).expect("Cannot unwrap token's key");
        // String extension is just placeholder!!

        let token_res = jsonwebtoken::decode::<TokenClaim<TokenVersion>>(token, &dk, &validation);

        match token_res {
            Ok(token) => Ok(token),
            Err(token_error) => Err(token_error.to_string()),
        }
    }
}

#[test]
fn test_validation() {
    let val = Validation::new(Algorithm::RS384);
    let csrf = utility::CsrfGuard::new();
    let ori_id = Uuid::new_v4();
    let acs = AccessToken::create_access_token::<utility::CsrfGuard>(
        ori_id.clone(),
        vec!["read"],
        Some(csrf),
    );

    let decoded_id = AccessToken::verify::<utility::CsrfGuard>(&acs.to_string())
        .unwrap()
        .claims
        .client_id;
    assert_eq!(ori_id, Uuid::parse_str(&decoded_id).unwrap());
}

// refreshing token process :
// verify the token
// -check the existence of user
// -compare token version
// regenerate access_token
//

// state could either represent error / success
pub trait TokenServer<State> {
    fn renew_token(&self, clid: &str, token: &str) -> Result<String, State>;
    fn revoked_token(&self, user_id: &str) -> bool;
}
