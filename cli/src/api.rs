// Wrapper functions to call api
use std::collections::HashMap;
use std::error::Error;
use reqwest::{Client, StatusCode, Url};
use common::model::password::Password;

pub struct Api {
	client: Client,
	base_url: Url,
}

impl Api {
	pub fn new(base_url: &str) -> Self {
		Self {
			client: Client::new(),
			base_url: Url::parse(base_url).unwrap(),
		}
	}

	pub async fn create_user(&self, public_key: String) -> Result<(), Box<dyn Error>> {
		let url = self.base_url.join("/users")?;

		let mut body = HashMap::new();
		body.insert("public_key", public_key);
		match self.client.post(url).json(&body).send().await {
			Ok(_) => Ok(()),
			Err(e) => Err(Box::new(e)),
		}
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

		// TODO: Actual auth
		let res = self.client.post(url)
			.header("Authorization", "Bearer 1234")
			.json(&body).send().await?;

		// TODO: Map more errors
		match res.status() {
			StatusCode::CREATED => Ok(()),
			_ => {
				let text = res.text().await.unwrap_or("".to_string());
				Err(format!("Error from API: {}", text).into())
			}
		}
	}

	pub async fn get_passwords(&self, public_key: String, site: String, _username: Option<String>) -> Result<Vec<Password>, Box<dyn Error>> {
		let url = self.base_url.join(&format!("/users/{}/passwords", public_key))?;

		// TODO: Actual auth
		let res = self.client.get(url)
			.header("Authorization", "Bearer 1234")
			.send().await?;

		match res.status() {
			StatusCode::OK => (),
			_ => {
				return Err("Error from API".into());
			}
		}

		let body = res.json::<Vec<Password>>().await?;

		let passwords = body.into_iter().filter(|p| p.site == site).collect();
		Ok(passwords)
	}
}