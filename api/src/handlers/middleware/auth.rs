use axum::extract::{Path, TypedHeader};
use axum::headers::authorization::{Authorization, Bearer};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::RequestPartsExt;
use std::collections::HashMap;
use passphrasex_common::api::verify_auth_token;

pub async fn only_user<B>(
    Path(params): Path<HashMap<String, String>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let user_id = params.get("user_id").ok_or(StatusCode::BAD_REQUEST)?;
    let (mut parts, body) = request.into_parts();

    let auth: TypedHeader<Authorization<Bearer>> = parts
        .extract()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    match verify_auth_token(user_id, auth.token()) {
        Ok(_) => (),
        Err(_) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let response = next.run(Request::from_parts(parts, body)).await;

    Ok(response)
}
