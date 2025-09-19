use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Messages is a enum that reference all messages type sft protocol can process
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Messages {
    AuthRequest { user: String, key: String },
    AuthResponse { ok: bool, msg: String },
    FileStart { filename: String, size: u64 },
    FileChunk { data: Vec<u8> },
    FileEnd,
    Progress { byte_received: u64, total_byte: u64 },
    Error { msg: String },
    Ack, // Acknowledgment of receipt
    Ping,
    Pong,
}

pub struct SFT {}

impl SFT {
    // -----------------------
    // Auth methods
    // -----------------------

    pub async fn auth(stream: &mut tokio::net::TcpStream) -> anyhow::Result<String> {
        SFT::send(
            stream,
            &Messages::AuthRequest {
                user: "pepe".to_string(),
                key: "oui".to_string(),
            },
        )
        .await?;
        let msg = SFT::recv(stream).await?;
        println!("debug: recv msg:\n {:#?}", msg);
        match msg {
            Messages::AuthResponse { ok: false, msg: m } => Err(anyhow::anyhow!(m)),
            Messages::AuthResponse { ok: true, msg: m } => Ok(m),
            _ => Err(anyhow::anyhow!("Internal server error")),
        }
    }

    pub async fn check_auth(
        stream: &mut tokio::net::TcpStream,
        user: &str,
        key: &str,
    ) -> anyhow::Result<()> {
        if user.is_empty() || key.is_empty() {
            SFT::send(
                stream,
                &Messages::AuthResponse {
                    ok: false,
                    msg: "auth failed invalid credential".to_string(),
                },
            )
            .await?;
            return Ok(());
        }
        SFT::send(
            stream,
            &Messages::AuthResponse {
                ok: true,
                msg: "connexion establish !".to_string(),
            },
        )
        .await?;
        Ok(())
    }

    // -----------------------
    // I/O methods
    // -----------------------

    pub async fn recv(stream: &mut tokio::net::TcpStream) -> anyhow::Result<Messages> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let lenght = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; lenght];
        stream.read_exact(&mut buf).await?;

        let msg: Messages = serde_json::from_slice(&buf)?;
        Ok(msg)
    }

    pub async fn send(stream: &mut tokio::net::TcpStream, msg: &Messages) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(msg)?;
        let len = (payload.len() as u32).to_be_bytes();

        stream.write_all(&len).await?;
        stream.write_all(&payload).await?;
        Ok(())
    }
}
