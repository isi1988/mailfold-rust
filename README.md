# mailfold-rust

Official Rust client SDK for [Mailfold](https://github.com/isi1988/Mailfold) — a
self-hosted webmail/admin backend. This crate wraps Mailfold's machine-to-machine
REST API (send mail, list folders/messages, read a message, delete, search,
fetch attachments, toggle flags) behind a small, typed `Client`.

This is the **official client SDK** for the
[isi1988/Mailfold](https://github.com/isi1988/Mailfold) project.

## Install

```
cargo add mailfold
```

or add it manually to `Cargo.toml`:

```toml
[dependencies]
mailfold = "0.1"
```

## Auth

Every request is authenticated with a per-mailbox API key (a bearer token
like `mf_live_<kid>_<secret>`) issued by your Mailfold instance. Treat it as
an opaque string — never parse it, just pass it to the client verbatim.

## Quickstart

```rust
use mailfold::{Client, SendMessage, Flag};

fn main() -> mailfold::Result<()> {
    let client = Client::new("https://real.mailfold.site", "mf_live_xxxx_yyyy")?;

    // Send a message. "From" is always the mailbox the key is bound to.
    client.send(
        &SendMessage::new()
            .to(["alice@example.com"])
            .subject("Hello from mailfold-rust")
            .text("Hi Alice!"),
    )?;

    // List folders.
    let folders = client.folders()?;
    for folder in &folders {
        println!("folder: {} {:?}", folder.name, folder.attributes);
    }

    // List recent messages in INBOX.
    let headers = client.messages(Some("INBOX"), Some(20))?;
    for h in &headers {
        println!("#{} {} - {}", h.uid, h.subject, h.date);
    }

    if let Some(first) = headers.first() {
        // Fetch the full message body.
        let full = client.message("INBOX", first.uid)?;
        println!("text body: {}", full.text);

        // Download the first attachment, if any.
        if !full.attachments.is_empty() {
            let att = client.attachment("INBOX", first.uid, 0)?;
            println!("downloaded {} bytes, content-type {:?}", att.bytes.len(), att.content_type);
        }

        // Mark it as read.
        client.set_flag("INBOX", first.uid, Flag::Seen, true)?;
    }

    // Search a folder.
    let results = client.search("INBOX", "invoice")?;
    println!("found {} messages", results.len());

    // Delete a message (careful, this is real).
    if let Some(h) = results.first() {
        client.delete_message("INBOX", h.uid)?;
    }

    Ok(())
}
```

## Error handling

Every fallible call returns `mailfold::Result<T>` (`Result<T, mailfold::Error>`).
API errors carry the HTTP status, the server's `"error"` message, and — for
429 responses — the `Retry-After` value in seconds:

```rust
match client.folders() {
    Ok(folders) => { /* ... */ }
    Err(mailfold::Error::Api { status, message, retry_after }) => {
        eprintln!("mailfold error {status}: {message} (retry_after={retry_after:?})");
    }
    Err(e) => eprintln!("transport error: {e}"),
}
```

## License

MIT, see [LICENSE](./LICENSE).
