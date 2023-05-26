use axum::extract::{TypedHeader, Path};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::headers::authorization::{Authorization, Bearer};
use axum::RequestPartsExt;

pub async fn only_user<B>(
	Path(user_id): Path<String>,
	request: Request<B>,
	next: Next<B>
) -> Result<Response, StatusCode> {
	let (mut parts, body) = request.into_parts();

	let auth: TypedHeader<Authorization<Bearer>> = parts.extract()
		.await.map_err(|_| StatusCode::UNAUTHORIZED)?;

	// TODO: Verify token

	println!("User id: {}", user_id);
	println!("Authorization: {}", auth.token());

	let response = next.run(Request::from_parts(parts, body)).await;

	Ok(response)
}
