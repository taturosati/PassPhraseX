use serde::{Deserialize, Serialize};

pub type RequestId = usize;

pub const FIRST_REQUEST_ID: RequestId = 1;

pub const INITIAL_REQUEST_ID: RequestId = FIRST_REQUEST_ID - 1;

pub fn next_request_id(last_request_id: RequestId) -> RequestId {
    last_request_id.wrapping_add(1).max(FIRST_REQUEST_ID)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestHeader {
    pub client_token: Option<String>,
}

impl RequestHeader {
    pub const fn new() -> Self {
        Self { client_token: None }
    }

    pub fn into_response(self, request_id: RequestId) -> ResponseHeader {
        let Self { client_token } = self;
        ResponseHeader {
            client_token,
            request_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request<T> {
    pub header: RequestHeader,
    pub payload: T,
}

impl<T> Request<T> {
    pub const fn new(payload: T) -> Self {
        Self {
            header: RequestHeader::new(),
            payload,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseHeader {
    pub client_token: Option<String>,
    pub request_id: RequestId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub header: ResponseHeader,
    pub payload: T,
}

/// App request message.
#[derive(Debug, Serialize, Deserialize)]
pub enum AppRequestPayload {
    GetOptionsInfo,
    GetStatus,
    Unlock {
        device_password: String,
    },
    Login {
        seed_phrase: String,
        device_password: String,
    },
    GetCredential {
        site: String,
        username: Option<String>,
    },
    AddCredential {
        site: String,
        username: String,
        password: String,
    },
}

pub type AppRequest = Request<AppRequestPayload>;

/// App response message.
#[derive(Debug, Serialize, Deserialize)]
pub enum AppResponsePayload {
    Status { is_logged_in: bool },
    OptionsInfo { version: String },
    Auth { error: Option<String> },
    Credential { username: String, password: String },
}

pub type AppResponse = Response<AppResponsePayload>;

/// Port-local request message.
#[derive(Debug, Serialize, Deserialize)]
pub enum PortRequestPayload {
    GetCredential { site: String },
}

pub type PortRequest = Request<PortRequestPayload>;

/// Port-local response message.
#[derive(Debug, Serialize, Deserialize)]
pub enum PortResponsePayload {
    Credential { username: String, password: String },
}

pub type PortResponse = Response<PortResponsePayload>;
