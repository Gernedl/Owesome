use leptos::prelude::*;
use leptos::server_fn::codec::JsonEncoding;
use leptos::server_fn::error::{FromServerFnError, ServerFnErrorErr};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
struct UsersRow {
    id: i64,
    username: String,
    password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginError {
    InvalidCredentials,
    // Technical errors (network, serialization, ...) that Leptos hands us.
    ServerError(ServerFnErrorErr),
}

impl FromServerFnError for LoginError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        LoginError::ServerError(value)
    }
}

#[server]
pub async fn login(username: String, password: String) -> Result<(), LoginError> {
    use crate::session::sign;
    use argon2::{
        password_hash::{phc::PasswordHash, PasswordVerifier},
        Argon2,
    };
    use axum::http::{header, HeaderValue};
    use axum_extra::extract::cookie::{Cookie, SameSite};

    fn server_error(e: impl std::fmt::Display) -> LoginError {
        LoginError::ServerError(ServerFnErrorErr::ServerError(e.to_string()))
    }

    let pool = expect_context::<sqlx::PgPool>();
    let result: Option<UsersRow> = sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&pool)
        .await
        .map_err(server_error)?;

    // check if it exists and unpack
    if result.is_none() {
        return Err(LoginError::InvalidCredentials);
    }
    let result = result.unwrap();

    // Hash the entered password with the salt stored in password_hash and compare.
    // Argon2 is deliberately slow, so keep it off the async worker threads.
    let stored_hash = result.password_hash.clone();
    let password_ok = tokio::task::spawn_blocking(move || {
        let Ok(stored_hash) = PasswordHash::new(&stored_hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &stored_hash)
            .is_ok()
    })
    .await
    .map_err(server_error)?;

    if !password_ok {
        return Err(LoginError::InvalidCredentials);
    }

    let payload = sign(result.id, &result.username);

    let cookie = Cookie::build(("session", payload))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        // .secure(true) // enable in production (HTTPS)
        .build();

    let res = expect_context::<leptos_axum::ResponseOptions>();
    res.append_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(server_error)?,
    );
    leptos_axum::redirect("/");
    Ok(())
}

#[server]
pub async fn get_current_user() -> Result<Option<String>, ServerFnError> {
    use crate::session::get_username;
    use axum_extra::extract::CookieJar;

    let jar: CookieJar = leptos_axum::extract().await?;
    if let Some(cookie) = jar.get("session") {
        return Ok(get_username(cookie.value()));
    }
    Ok(None)
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use axum::http::{header, HeaderValue};
    use axum_extra::extract::cookie::Cookie;

    let mut expired = Cookie::build(("session", "")).path("/").build();
    expired.make_removal();
    let res = expect_context::<leptos_axum::ResponseOptions>();
    res.append_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&expired.to_string())?,
    );
    leptos_axum::redirect("/login");
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegisterError {
    InvalidCode,
    UsernameTaken,
    // Technical errors (network, serialization, ...) that Leptos hands us.
    ServerError(ServerFnErrorErr),
}

impl FromServerFnError for RegisterError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        RegisterError::ServerError(value)
    }
}

#[server]
pub async fn register(
    username: String,
    password: String,
    code: String,
) -> Result<(), RegisterError> {
    use crate::registration::check_code;
    use argon2::{password_hash::PasswordHasher, Argon2};

    // Wraps any technical error (database, hashing, ...) for the client.
    fn server_error(e: impl std::fmt::Display) -> RegisterError {
        RegisterError::ServerError(ServerFnErrorErr::ServerError(e.to_string()))
    }

    if !check_code(code.as_str()) {
        return Err(RegisterError::InvalidCode);
    }

    // Argon2 is deliberately slow, so keep it off the async worker threads.
    let password_hash = tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
    })
    .await
    .map_err(server_error)?
    .map_err(server_error)?;

    let pool = expect_context::<sqlx::PgPool>();
    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES ($1, $2)")
        .bind(&username)
        .bind(&password_hash)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => {
            leptos_axum::redirect("/login");
            Ok(())
        }
        // The UNIQUE constraint on users.username rejected the insert.
        Err(e)
            if e.as_database_error()
                .is_some_and(|db| db.is_unique_violation()) =>
        {
            Err(RegisterError::UsernameTaken)
        }
        Err(e) => Err(server_error(e)),
    }
}

#[server]
pub async fn get_code() -> Result<String, ServerFnError> {
    use crate::registration::get_code;
    Ok(get_code())
}
