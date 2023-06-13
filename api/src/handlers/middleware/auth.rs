use std::collections::HashMap;
use std::time::SystemTime;
use axum::extract::{TypedHeader, Path};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::headers::authorization::{Authorization, Bearer};
use axum::RequestPartsExt;
use passphrasex_common::crypto::asymmetric::{public_key_from_base64, verify};
use passphrasex_common::crypto::common::EncryptedValue;

const SECS_TOLERANCE: u64 = 3;

pub async fn only_user<B>(
	Path(params): Path<HashMap<String, String>>,
	request: Request<B>,
	next: Next<B>
) -> Result<Response, StatusCode> {
	let user_id = params.get("user_id").ok_or(StatusCode::BAD_REQUEST)?;
	let (mut parts, body) = request.into_parts();

	let auth: TypedHeader<Authorization<Bearer>> = parts.extract()
		.await.map_err(|_| StatusCode::UNAUTHORIZED)?;

	let enc = EncryptedValue::from(auth.token().to_string());
	let public_key = public_key_from_base64(&user_id);
	let dec = match verify(&public_key, enc) {
		Ok(dec) => dec,
		Err(_) => return Err(StatusCode::UNAUTHORIZED)
	};

	let dec_time: u64 = dec.parse().or_else(|_| Err(StatusCode::UNAUTHORIZED))?;

	// dec should be current timestamp in seconds (with some tolerance)
	let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
		.expect("Time went backwards").as_secs();

	if time - dec_time > SECS_TOLERANCE {
		return Err(StatusCode::UNAUTHORIZED);
	}

	let response = next.run(Request::from_parts(parts, body)).await;

	Ok(response)
}
