use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize)]
pub struct SendMessage {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

impl SendMessage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to(mut self, addrs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.to = addrs.into_iter().map(Into::into).collect();
        self
    }

    pub fn cc(mut self, addrs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.cc = addrs.into_iter().map(Into::into).collect();
        self
    }

    pub fn bcc(mut self, addrs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.bcc = addrs.into_iter().map(Into::into).collect();
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendResponse {
    pub status: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Folder {
    pub name: String,
    #[serde(default)]
    pub attributes: Vec<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Address {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageHeader {
    pub uid: u32,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub from: Vec<Address>,
    #[serde(default)]
    pub to: Vec<Address>,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub seen: bool,
    #[serde(default)]
    pub size: u32,
    #[serde(default)]
    pub preview: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Attachment {
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub size: i64,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub uid: u32,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub from: Vec<Address>,
    #[serde(default)]
    pub to: Vec<Address>,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub seen: bool,
    #[serde(default)]
    pub size: u32,
    #[serde(default)]
    pub preview: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub html: String,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Flag {
    Seen,
    Flagged,
    Answered,
    Deleted,
    Draft,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Returned by the raw-bytes attachment endpoint alongside the body, since the
/// response isn't JSON and headers are otherwise lost to the caller.
#[derive(Debug, Clone, Default)]
pub struct AttachmentData {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
    pub filename: Option<String>,
}
