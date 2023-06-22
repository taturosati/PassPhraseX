use mongodb::error::Result;
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use mongodb::{Client, Collection};
use std::env;

pub struct DatabaseConfig {
    pub uri: String,
}

impl DatabaseConfig {
    pub fn new() -> Self {
        let uri = env::var("MONGODB_URI").unwrap_or("mongodb://localhost:27017".to_string());

        Self { uri }
    }

    pub async fn into_client(self) -> Result<Client> {
        let mut client_options = ClientOptions::parse(self.uri).await?;
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);
        Client::with_options(client_options)
    }
}

pub trait GetCollection {
    fn get_collection<T>(&self, name: &str) -> Collection<T>;
}

impl GetCollection for Client {
    fn get_collection<T>(&self, name: &str) -> Collection<T> {
        let db = self.database("passphrasex");
        db.collection(name)
    }
}
