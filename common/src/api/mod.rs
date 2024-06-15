mod time;

// Wrapper functions to call api
use crate::crypto::asymmetric::{KeyPair, verify, verifying_key_from_base64};
use crate::model::password::Password;
use anyhow::format_err;
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use reqwest::{Client, Response, StatusCode, Url};
use std::collections::HashMap;
use std::env;
use time::SystemTime;

#[derive(Clone)]
pub struct Api {
    client: Client,
    base_url: Url,
    key_pair: KeyPair,
}

impl Api {
    pub fn new(key_pair: KeyPair) -> Self {
        let base_url =
            env::var("API_URI").unwrap_or("https://api.passphrasex.srosati.xyz".to_string());

        Self {
            client: Client::new(),
            base_url: Url::parse(&base_url).unwrap(),
            key_pair,
        }
    }

    pub async fn create_user(&self, public_key: String) -> anyhow::Result<()> {
        let url = self.base_url.join("/users")?;

        let mut body = HashMap::new();
        body.insert("public_key", public_key);
        let res = self.client.post(url).json(&body).send().await?;
        validate_response(res, StatusCode::CREATED).await
    }

    pub async fn add_password(&self, public_key: String, password: Password) -> anyhow::Result<()> {
        let url = self
            .base_url
            .join(&format!("/users/{}/passwords", public_key))?;

        let mut body = HashMap::new();
        body.insert("_id", password._id);
        body.insert("user_id", password.user_id);
        body.insert("site", password.site);
        body.insert("username", password.username);
        body.insert("password", password.password);

        let res = self
            .client
            .post(url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        validate_response(res, StatusCode::CREATED).await
    }

    pub async fn get_passwords(&self, public_key: String) -> anyhow::Result<Vec<Password>> {
        let url = self
            .base_url
            .join(&format!("/users/{}/passwords", public_key))?;

        let res = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => (),
            _ => {
                return Err(format_err!("Error from API: {}", res.text().await?));
            }
        }

        let body = res.json::<Vec<Password>>().await?;
        Ok(body)
    }

    pub async fn edit_password(
        &self,
        public_key: String,
        password_id: String,
        password: String,
    ) -> anyhow::Result<()> {
        let url = self.base_url.join(&format!(
            "/users/{}/passwords/{}/password",
            public_key, password_id
        ))?;

        let res = self
            .client
            .put(url)
            .header("Authorization", self.auth_header())
            .body(password)
            .send()
            .await?;

        validate_response(res, StatusCode::NO_CONTENT).await
    }

    pub async fn delete_password(
        &self,
        public_key: String,
        password_id: String,
    ) -> anyhow::Result<()> {
        let url = self
            .base_url
            .join(&format!("/users/{}/passwords/{}", public_key, password_id))?;

        let res = self
            .client
            .delete(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        validate_response(res, StatusCode::NO_CONTENT).await
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.auth_token())
    }

    fn auth_token(&self) -> String {
        let time = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let signature = self.key_pair.sign(&time.to_string());
        let signature = URL_SAFE.encode(signature);
        format!("{};{}", time, signature)
    }
}

async fn validate_response(res: Response, status_code: StatusCode) -> anyhow::Result<()> {
    if res.status() != status_code {
        let text = res.text().await?;
        return Err(format_err!("Error from API: {}", text));
    }

    Ok(())
}

pub fn verify_auth_token(public_key: &str, token: &str) -> anyhow::Result<()> {
    let (time, signature) = token.split_once(';').ok_or(format_err!("Invalid token"))?;
    let time = time.parse::<u64>()?;
    let now = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    if now - time > 60 {
        return Err(format_err!("Token expired"));
    }

    let signature = URL_SAFE.decode(signature.as_bytes())?;
    let verifying_key = verifying_key_from_base64(public_key)?;
    match verify(verifying_key, time.to_string().as_bytes(), signature.as_slice()) {
        Ok(_) => (),
        Err(err) => {
            println!("{:?}", err);
            return Err(format_err!("Invalid token: {}", err));
        }
    }

    Ok(())
}
