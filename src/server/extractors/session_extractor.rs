use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use axum::Extension;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use tracing::error;
use uuid::Uuid;

use crate::server::error::Error;
use crate::server::services::Services;

pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

/// Scoped to the users API so the browser only sends it to signin, refresh and signout.
pub fn refresh_token_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build(REFRESH_TOKEN_COOKIE, value)
        .http_only(true)
        .same_site(SameSite::Strict)
        .path("/api/v1/users")
        .secure(secure)
        .finish()
}

fn refresh_token_from_headers(headers: &HeaderMap) -> Option<String> {
    CookieJar::from_headers(headers)
        .get(REFRESH_TOKEN_COOKIE)
        .map(|cookie| cookie.value().to_string())
}

/// Extracts the JWT from the cookie token header.
pub struct SessionExtractor(pub Uuid, pub String);

#[async_trait]
impl<S> FromRequestParts<S> for SessionExtractor
where
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(services): Extension<Services> = Extension::from_request_parts(parts, state)
            .await
            .map_err(|err| Error::InternalServerErrorWithContext(err.to_string()))?;

        let refresh_token = refresh_token_from_headers(&parts.headers).ok_or(Error::Unauthorized)?;

        let session_id = services
            .jwt_util
            .get_session_id_from_token(refresh_token.clone())
            .map_err(|err| {
                error!("could not validate session ID from token: {:?}", err);
                Error::Unauthorized
            })?;

        Ok(SessionExtractor(session_id, refresh_token))
    }
}

#[cfg(test)]
mod tests {
    use axum::http::header::COOKIE;
    use axum::http::HeaderValue;

    use super::*;

    fn headers(cookie: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str(cookie).unwrap());
        headers
    }

    #[test]
    fn finds_refresh_token_among_other_cookies() {
        let headers = headers("theme=dark; refresh_token=abc.def; lang=id");
        assert_eq!(refresh_token_from_headers(&headers).as_deref(), Some("abc.def"));
    }

    #[test]
    fn returns_none_without_refresh_token() {
        assert!(refresh_token_from_headers(&headers("theme=dark")).is_none());
        assert!(refresh_token_from_headers(&HeaderMap::new()).is_none());
    }

    #[test]
    fn refresh_cookie_is_http_only_and_scoped() {
        let cookie = refresh_token_cookie("abc".into(), false).to_string();
        assert_eq!(cookie, "refresh_token=abc; HttpOnly; SameSite=Strict; Path=/api/v1/users");
        assert!(refresh_token_cookie("abc".into(), true).to_string().contains("; Secure"));
    }
}
