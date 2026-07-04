//! Official Rust client SDK for [Mailfold](https://github.com/isi1988/Mailfold),
//! a self-hosted webmail/admin backend. Wraps its per-mailbox API-key-authenticated
//! REST surface: sending mail, listing folders/messages, reading a message,
//! deleting, searching, fetching attachments, and toggling flags.
//!
//! See the crate README for a full quickstart.

mod client;
mod error;
mod types;

pub use client::Client;
pub use error::{Error, Result};
pub use types::{
    Address, Attachment, AttachmentData, Flag, Folder, Message, MessageHeader, SendMessage,
    SendResponse, StatusResponse,
};
