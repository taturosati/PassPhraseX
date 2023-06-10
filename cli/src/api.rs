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
	pub fn new(base_url: &str, key_pair: KeyPair) -> Self {
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
		site: String,
		username: String,
		password: String
	) -> Result<(), Box<dyn Error>> {
		let url = self.base_url.join(&format!("/users/{}/passwords", public_key))?;

		let mut body = HashMap::new();
		body.insert("site", site);
		body.insert("username", username);
		body.insert("password", password);

		let res = self.client.post(url)
			.header("Authorization", self.auth_header())
			.json(&body).send().await?;

		validate_response(res, StatusCode::CREATED).await
	}

	pub async fn get_passwords(&self, public_key: String, site: Option<String>, _username: Option<String>) -> Result<Vec<Password>, Box<dyn Error>> {
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

		match site {
			Some(site) => {
				let passwords = body.into_iter().filter(|p| p.site == site).collect();
				Ok(passwords)
			},
			None => Ok(body)
		}
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