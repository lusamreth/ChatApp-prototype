/// Util funtions
use rand::{rngs::OsRng,RngCore};
use super::jwt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize,Serialize};

pub fn extract_password(pass: String) {
    let pass_rule = regex::Regex::new(r#""#).expect("bad regex!");
    if pass_rule.is_match(&pass) {
        print!("true")
    } else {
        print!("fails")
    }
}

pub fn timestamp_now() -> Duration {
    let sys = SystemTime::now();
    let now = sys.duration_since(UNIX_EPOCH).expect("Time went backward!");
    return now;
}
pub fn sanitize_text(text: &str) -> bool {
    let excluded = regex::Regex::new(r"[ $ / # ? * \\ \[ \] ]").expect("bad regex!");
    excluded.is_match(text)
}

pub fn build_extract_backlash(prefix: &str, len: usize) -> Box<dyn Fn(&str) -> Option<String>> {
    let match_regex = format!(r#"({})(/)(.*)+"#, prefix);
    let regx = regex::Regex::new(&match_regex).expect("bad regex!");
    Box::new(move |text| {
        println!("testxt {}", text);
        match regx.clone().captures(text) {
            Some(groups) => {
                println!("cap {:#?}", groups);
                let param = groups.get(3).map_or("", |m| m.as_str());
                if param.len() < len {
                    Some(param.to_string())
                } else {
                    None
                }
            }
            None => None,
        }
        // scan the param group 3
    })
}
// recommend by rfc 8018 : IC >= 1000
const DefaultIterationCount: u32 = 3600;

type Base64 = String;
pub fn hash_sha256(input: String) -> Base64 {
    // pbkdf2 is based on sha256
    let hash = pbkdf2::pbkdf2_simple(&input, DefaultIterationCount).expect("Randomization Error!");
    return hash;
}

pub fn compare_sha256(input: &str, hashed: &str) -> bool {
    let comp = pbkdf2::pbkdf2_check(input, hashed).ok();
    return comp.is_some();
}


#[derive(Serialize, Deserialize)]
pub struct CsrfGuard{
    #[serde(default)]
    csrf: String, //random base 64 string!!
}
impl CsrfGuard {

    pub fn new() -> Self{
        CsrfGuard {
            csrf:CsrfGuard::generate_csrf_token()
        }
    }

    pub fn token(&self) -> String {
        return self.csrf.clone();
    }

    fn generate_csrf_token() -> String{
        // 32 bytes value
        let mut k = [0u8;32];
        OsRng.fill_bytes(&mut k);
        let b64_k = base64::encode(&mut k);
        return b64_k;

    }

}

#[test]
fn test_guard(){
    let tk = CsrfGuard::generate_csrf_token();
    println!("tk {:#?}",tk);
}
