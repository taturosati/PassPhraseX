use mongodb::Client;
use mongodb::options::ClientOptions;
use mongodb::error::Result;

pub struct DatabaseConfig {
    pub uri: String
}

impl DatabaseConfig {
    pub fn new() -> Self {
        let uri = "mongodb://localhost:27017".to_string();
        Self {
            uri
        }
    }

    pub async fn into_client(self) -> Result<Client> {
        let client_options = ClientOptions::parse(self.uri).await?;
        Client::with_options(client_options)
    }
}