// Wrapper functions to call api
use std::collections::HashMap;
use std::error::Error;
use std::time::SystemTime;
use reqwest::{Client, Response, StatusCode, Url};
use common::crypto::asymmetric::KeyPair;
use common::model::password::Password;

pub struct Api {
	client: Client,
	base_url: Url,
	key_pair: KeyPair
}

impl Api {
	pub fn new(key_pair: KeyPair) -> Self {
		let base_url = "http://localhost:3000"; // TODO: Read from env
		Self {
			client: Client::new(),
			base_url: Url::parse(base_url).unwrap(),
			key_pair
		}
	}

	pub async fn create_user(&self, public_key: String) -> Result<(), Box<dyn Error>> {
		let url = self.base_url.join("/users")?;

		let mut body = HashMap::new();
		body.insert("public_key", public_key);
		let res = self.client.post(url).json(&body).send().await?;
		validate_response(res, StatusCode::CREATED).await
	}

	pub async fn add_password(
		&self,
		public_key: String,
		password: Password
	) -> Result<(), Box<dyn Error>> {
		let url = self.base_url.join(&format!("/users/{}/passwords", public_key))?;

		let mut body = HashMap::new();
		body.insert("_id", password._id);
		body.insert("site", password.site);
		body.insert("username", password.username);
		body.insert("password", password.password);

		let res = self.client.post(url)
			.header("Authorization", self.auth_header())
			.json(&body).send().await?;

		validate_response(res, StatusCode::CREATED).await
	}

	pub async fn get_passwords(&self, public_key: String) -> Result<Vec<Password>, Box<dyn Error>> {
		let url = self.base_url.join(&format!("/users/{}/passwords", public_key))?;

		let res = self.client.get(url)
			.header("Authorization", self.auth_header())
			.send().await?;

		match res.status() {
			StatusCode::OK => (),
			_ => {
				return Err(format!("Error from API: {}", res.text().await?).into());
			}
		}

		let body = res.json::<Vec<Password>>().await?;
		Ok(body)
	}

	pub async fn edit_password(&self, public_key: String, password_id: String, password: String) -> Result<(), Box<dyn Error>> {
		let url = self.base_url.join(&format!("/users/{}/passwords/{}/password", public_key, password_id))?;

		let res = self.client.put(url)
			.header("Authorization", self.auth_header())
			.body(password).send().await?;

		validate_response(res, StatusCode::NO_CONTENT).await
	}

	fn auth_header(&self) -> String {
		format!("Bearer {}", self.auth_token())
	}

	fn auth_token(&self) -> String {
		let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
			.unwrap().as_secs();
		self.key_pair.sign(&time.to_string()).to_string()
	}
}


async fn validate_response(res: Response, status_code: StatusCode) -> Result<(), Box<dyn Error>> {
	if res.status() != status_code {
		let text = res.text().await?;
		return Err(format!("Error from API: {}", text).into());
	}

	Ok(())
}