use crate::{error::Error, Result, WebResult};
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use std::fmt;
use warp::{filters::header::headers_cloned, http::header::{HeaderMap, HeaderValue, AUTHORIZATION}, reject, Rejection, Filter};

use crate::WebResult;

const BEARER: &str = "Bearer";
const JWT_SECRET: &[u8] = b"secret";

#[derive(Clone, PartialEq)]
pub enum Role{
    User,
    Admin,
}

impl Role{
    pub fn from_str(role: &str) -> Role{
        match role{
            "user" => Role::User,
            "admin" => Role::Admin,
            _ => None,
        }
    }
}

impl fmt::Display for Role{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        match self{
            Role::User => write!(f, "User"),
            Role::Admin => write!(f, "Admin"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims{
    sub: String,
    role: String,
    exp: usize,
}

pub fn with_auth(role: Role) -> impl Filter<Extract = (String,), Error = Rejection> + Clone{
    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| (role.clone(), headers))
        .and_then(authorize)
}


pub fn create_token(uid: &str, role: &Role) -> Result<String>{
    let exp = Utc::now().checked_add_signed(chrono::Duration::seconds(60)).expect("valid timestamp").timestamp();
    let claims = Claims{
        sub: uid.to_owned(),
        role: role.to_string(),
        exp: exp as usize,
    };
    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET)).map_err(|_| Error::JWTTokenCreationError)
}

async fn authorize((role, headers): (Role, HeaderMap<HeaderValue>)) -> WebResult<String>{
    match jwt_from_header(&headers){
        Ok(jwt) => {
            let decoded = decode::<Claims>(&jwt, &DecodingKey::from_secret(JWT_SECRET), &Validation::new(Algorithm::HS512))
                .map_err(|_| reject::custom(Error::JWTTokenError))?;

            if role == Role::Admin && Role::from_str(&decoded.claims.role) != Role::Admin{
                return Err(reject::custom(Error::Unauthorized));
            }

            Ok(decoded.claims.sub)
        }
        Err(e) => return Err(reject::custom(e)),
    }
}

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, Rejection>{
    let header = match headers.get(AUTHORIZATION){
        Some(header) => header,
        None => Err(Error::NoAuthHeaderError),
    };

    let auth_header = match std::str::from_utf8(header.as_bytes()){
        Ok(s) => s,
        Err(_) => return Err(Error::NoAuthHeaderError),
    };

    if !auth_header.starts_with(BEARER){
        return Err(Error::InvalidAuthError);
    }

    Ok(auth_header.trim_start_matches(BEARER).to_owned())


}