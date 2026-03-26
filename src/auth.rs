use crate::err;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::Form;
use axum_login::{AuthSession, AuthUser, AuthnBackend, UserId};
use maud::Markup;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[derive(Debug, Clone)]
pub struct User {
    id: u32,
    pw_hash: Vec<u8>,
}

impl AuthUser for User {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash
    }
}

#[derive(Clone, Default)]
pub struct BackendRudimentary {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Debug, Snafu)]
pub enum AuthError {
    #[snafu(display("Invalid credentials"))]
    InvalidCredentials,

    #[snafu(display("User not found"))]
    UserNotFound,
}

pub const DEFAULT_USERNAME: &'static str = "admin";
pub const DEFAULT_PASSWORD: &'static str = "admin";

pub const DEFAULT_USER: User = User {
    id: 1,
    pw_hash: vec![],
};

#[async_trait]
impl AuthnBackend for BackendRudimentary {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<User>, AuthError> {
        if credentials.username != DEFAULT_USERNAME || credentials.password != DEFAULT_PASSWORD {
            return Ok(None);
        }
        Ok(Some(DEFAULT_USER))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, AuthError> {
        if user_id != &DEFAULT_USER.id {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(Some(DEFAULT_USER))
    }
}

pub async fn page_login_check(
    mut auth_session: AuthSession<BackendRudimentary>,
    Form(login_form): Form<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.backend.authenticate(login_form.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to("/login_success").into_response()
}
