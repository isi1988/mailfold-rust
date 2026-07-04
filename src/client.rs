use reqwest::blocking::{Client as HttpClient, Response};
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

use crate::error::{Error, Result};
use crate::types::*;

/// Client for a single Mailfold mailbox, identified by its API key.
///
/// `base_url` points at the Mailfold instance root, e.g. `https://real.mailfold.site`
/// (no default is hardcoded — every self-hosted instance has its own host).
pub struct Client {
    base_url: String,
    api_key: String,
    http: HttpClient,
}

impl Client {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Result<Self> {
        let http = HttpClient::builder().build()?;
        Ok(Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            http,
        })
    }

    /// Use a pre-built `reqwest::blocking::Client` (e.g. to set custom timeouts/proxies).
    pub fn with_http_client(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        http: HttpClient,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            http,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn request(&self, method: Method, path: &str) -> reqwest::blocking::RequestBuilder {
        self.http
            .request(method, self.url(path))
            .bearer_auth(&self.api_key)
    }

    fn handle_error(resp: Response) -> Error {
        let status = resp.status().as_u16();
        let retry_after = resp
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        #[derive(Deserialize)]
        struct ErrorBody {
            error: String,
        }

        let message = match resp.text() {
            Ok(text) => serde_json::from_str::<ErrorBody>(&text)
                .map(|b| b.error)
                .unwrap_or(text),
            Err(_) => "unknown error".to_string(),
        };

        Error::Api {
            status,
            message,
            retry_after,
        }
    }

    fn send_json<T: DeserializeOwned>(
        &self,
        builder: reqwest::blocking::RequestBuilder,
    ) -> Result<T> {
        let resp = builder.send()?;
        if !resp.status().is_success() {
            return Err(Self::handle_error(resp));
        }
        let text = resp.text()?;
        serde_json::from_str(&text).map_err(Error::Decode)
    }

    /// POST /api/v1/mail/send (scope: mail:send)
    pub fn send(&self, message: &SendMessage) -> Result<SendResponse> {
        let builder = self.request(Method::POST, "/api/v1/mail/send").json(message);
        self.send_json(builder)
    }

    /// GET /api/v1/mail/folders (scope: mail:read)
    pub fn folders(&self) -> Result<Vec<Folder>> {
        let builder = self.request(Method::GET, "/api/v1/mail/folders");
        self.send_json(builder)
    }

    /// GET /api/v1/mail/messages (scope: mail:read)
    ///
    /// `folder` defaults server-side to `INBOX` and `limit` to 50 when `None`.
    pub fn messages(
        &self,
        folder: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<MessageHeader>> {
        let mut builder = self.request(Method::GET, "/api/v1/mail/messages");
        let mut query = Vec::new();
        if let Some(folder) = folder {
            query.push(("folder", folder.to_string()));
        }
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        builder = builder.query(&query);
        self.send_json(builder)
    }

    /// GET /api/v1/mail/message (scope: mail:read)
    pub fn message(&self, folder: &str, uid: u32) -> Result<Message> {
        let builder = self
            .request(Method::GET, "/api/v1/mail/message")
            .query(&[("folder", folder), ("uid", &uid.to_string())]);
        self.send_json(builder)
    }

    /// DELETE /api/v1/mail/message (scope: mail:write)
    pub fn delete_message(&self, folder: &str, uid: u32) -> Result<StatusResponse> {
        let builder = self
            .request(Method::DELETE, "/api/v1/mail/message")
            .json(&json!({ "folder": folder, "uid": uid }));
        self.send_json(builder)
    }

    /// GET /api/v1/mail/search (scope: mail:read)
    pub fn search(&self, folder: &str, q: &str) -> Result<Vec<MessageHeader>> {
        let builder = self
            .request(Method::GET, "/api/v1/mail/search")
            .query(&[("folder", folder), ("q", q)]);
        self.send_json(builder)
    }

    /// GET /api/v1/mail/attachment (scope: mail:read)
    ///
    /// Unlike every other endpoint this returns raw bytes, not JSON, so the
    /// response is read directly and paired with the content-type/filename
    /// headers the server sets rather than being deserialized.
    pub fn attachment(&self, folder: &str, uid: u32, index: u32) -> Result<AttachmentData> {
        let builder = self
            .request(Method::GET, "/api/v1/mail/attachment")
            .query(&[
                ("folder", folder.to_string()),
                ("uid", uid.to_string()),
                ("index", index.to_string()),
            ]);
        let resp = builder.send()?;
        if !resp.status().is_success() {
            return Err(Self::handle_error(resp));
        }

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let filename = resp
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_filename);

        let bytes = resp.bytes()?.to_vec();
        Ok(AttachmentData {
            bytes,
            content_type,
            filename,
        })
    }

    /// POST /api/v1/mail/flag (scope: mail:write)
    pub fn set_flag(&self, folder: &str, uid: u32, flag: Flag, set: bool) -> Result<StatusResponse> {
        #[derive(Serialize)]
        struct FlagRequest<'a> {
            folder: &'a str,
            uid: u32,
            flag: Flag,
            set: bool,
        }
        let builder = self
            .request(Method::POST, "/api/v1/mail/flag")
            .json(&FlagRequest {
                folder,
                uid,
                flag,
                set,
            });
        self.send_json(builder)
    }
}

fn parse_filename(content_disposition: &str) -> Option<String> {
    content_disposition.split(';').find_map(|part| {
        let part = part.trim();
        let value = part
            .strip_prefix("filename*=")
            .or_else(|| part.strip_prefix("filename="))?;
        Some(value.trim_matches('"').to_string())
    })
}
