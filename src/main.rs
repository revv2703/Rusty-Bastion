use auth::{with_auth, Role};
use error::Error::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use warp::{reject, reply, Filter, Rejection, Reply};

mod auth;
mod error;

type Result<T> = std::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;

#[derive(Clone)]
pub struct User{
    pub uid: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct LoginRequest{
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse{
    pub token: String,
}

#[tokio::main]
async fn main(){
    let users = Arc::new(init_users());

    let login_route = warp::path!("login")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_users(users.clone()))
        .and_then(login_handler);

        let user_route = warp::path!("user")
            .and(with_auth(Role::User))
            .and_then(user_handler)

        let admin_route = warp::path!("admin")
            .and(with_auth(Role::Admin))
            .and_then(admin_handler);

        let routes = login_route.or(user_route).or(admin_route).recover(error::handle_rejection);

        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone{
    warp::any().map(move || users.clone())
}

pub async fn login_handler(users: Users, body: LoginRequest) -> WebResult<impl Reply>{

    match users
        .iter()
        .find(|(_uid, user)| user.email == body.email && user.password == body.password){
            Some((uid, user)) => {
                let token = auth::create_token(&uid, &Role::from_str(&user.role))
                    .map_err(|_| reject::custom(AuthError))?;
                Ok(reply::json(&LoginResponse { token }))
            }
            None => Err(reject::custom(WrongCredentialsError)),
        }
}

pub async fn user_handler(uid: String) -> WebResult<impl Reply>{
    Ok(format!("Greetings, User {}", uid))
}

pub async fn admin_handler(uid: String) -> WebResult<impl Reply>{
    Ok(format!("Greetings, Admin {}", uid))
}

fn init_users() -> HashMap<String, User>{
    let mut map = HashMap::new();
    map.insert(
        "1".to_string(),
        User{
            uid: "1".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            role: "User".to_string(),
        },
    );
    map.insert(
        "2".to_string(),
        User{
            uid: "2".to_string(),
            email: "admin@example.com".to_string(),
            password: "password".to_string(),
            role: "Admin".to_string(),
        },
    );
    map
}