use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use {{crate_name}}::api::rate_limit::RateLimiter;
use {{crate_name}}::api::{self, state::AppState};
use {{crate_name}}::application::auth::TokenIssuer;
use {{crate_name}}::infrastructure::auth::JwtTokenService;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

const SECRET: &str = "test-secret";

fn app_without_database() -> Router {
    api::router(AppState::new(None, SECRET), &[])
}

fn json_post(path: &str, body: &str) -> Request<Body> {
    Request::post(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_answers_ok() {
    let response = app_without_database()
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn malformed_json_uses_the_error_format() {
    let response = app_without_database()
        .oneshot(json_post("/api/v1/auth/login", "{not json"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_json(response).await["code"], "validation_error");
}

#[tokio::test]
async fn register_without_database_is_a_configuration_error() {
    let response = app_without_database()
        .oneshot(json_post(
            "/api/v1/auth/register",
            r#"{"display_name":"A","email":"a@example.com","password":"long-enough-password"}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body_json(response).await["code"], "configuration_error");
}

#[tokio::test]
async fn me_requires_a_bearer_token() {
    let response = app_without_database()
        .oneshot(Request::get("/api/v1/auth/me").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_json(response).await["code"], "unauthorized");
}

#[tokio::test]
async fn me_rejects_an_invalid_token() {
    let response = app_without_database()
        .oneshot(
            Request::get("/api/v1/auth/me")
                .header(header::AUTHORIZATION, "Bearer not-a-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_with_valid_token_reaches_the_database_check() {
    let token = JwtTokenService::new(SECRET)
        .issue_access_token("user-1")
        .unwrap();

    let response = app_without_database()
        .oneshot(
            Request::get("/api/v1/auth/me")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn login_is_rate_limited_per_client() {
    let state =
        AppState::new(None, SECRET).with_rate_limiter(RateLimiter::new(2, Duration::from_secs(60)));
    let app = api::router(state, &[]);
    let client: SocketAddr = "10.0.0.1:5000".parse().unwrap();

    let mut statuses = Vec::new();
    for _ in 0..3 {
        let mut request = json_post("/api/v1/auth/login", r#"{"email":"a@b.c","password":"x"}"#);
        request.extensions_mut().insert(ConnectInfo(client));
        statuses.push(app.clone().oneshot(request).await.unwrap().status());
    }

    assert_eq!(
        statuses,
        vec![
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::TOO_MANY_REQUESTS
        ]
    );
}
