use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_MESSAGE_SIZE: u64 = 10 * 1024 * 1024; // 10Mb

/// Messages is a enum that reference all messages type sft protocol can process
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Messages {
    AuthRequest {
        user: String,
        key: String,
    },
    AuthResponse {
        ok: bool,
        msg: String,
    },
    SessionInit {
        file_count: usize,
        filenames: Vec<String>,
    },
    SessionReady {
        ports: Vec<u16>,
    },
    FileStart {
        filename: String,
        size: u64,
    },
    FileChunk {
        data: Vec<u8>,
    },
    FileEnd,
    Progress {
        byte_received: u64,
        total_byte: u64,
    },
    Error {
        msg: String,
    },
    Ack, // Acknowledgment of receipt
    Close,
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

        if lenght > MAX_MESSAGE_SIZE as usize {
            anyhow::bail!("message to large: {lenght}");
        }

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

    pub async fn sendf(
        stream: &mut tokio::net::TcpStream,
        path: &str,
        mp: &MultiProgress,
    ) -> anyhow::Result<()> {
        let file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let size = metadata.len();

        let filename = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        SFT::send(
            stream,
            &Messages::FileStart {
                filename: filename.clone(),
                size,
            },
        )
        .await?;

        stream_file_content(file, stream, size, &filename, mp).await?;
        SFT::send(stream, &Messages::FileEnd).await?;
        loop {
            let resp = SFT::recv(stream).await?;
            match resp {
                Messages::Ack => {
                    // println!("File upload with success !");
                    break;
                }
                Messages::Error { msg } => anyhow::bail!("Internal server error: {msg}"),
                _ => {
                    println!("debug: server response : {:#?}", resp);
                    anyhow::bail!("Unexpected Response");
                }
            }
        }
        Ok(())
    }

    // TODO: send Progress only every 5/6 Mb to synch with client progress (need to increase speed)
    pub async fn recvf(stream: &mut tokio::net::TcpStream, filename: &str) -> anyhow::Result<()> {
        let path = format!("uploads/{filename}");
        let mut file = tokio::fs::File::create(path).await?;

        loop {
            match SFT::recv(stream).await? {
                Messages::FileChunk { data } => {
                    file.write_all(&data).await?;
                }
                Messages::FileEnd => {
                    drop(file);
                    println!("debug: recv EOF");
                    SFT::send(stream, &Messages::Ack).await?;
                    break;
                }
                _ => {
                    println!("debug: recv unexpected signal");
                    SFT::send(
                        stream,
                        &Messages::Error {
                            msg: "Unexpected message".into(),
                        },
                    )
                    .await?;
                    println!("error: unexpected messages");
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn ping(stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
        println!("ping");
        SFT::send(stream, &Messages::Ping).await?;
        match SFT::recv(stream).await? {
            Messages::Pong => Ok(()),
            _ => Err(anyhow::anyhow!("unexpected response")),
        }
    }

    pub async fn pong(stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
        println!("pong");
        SFT::send(stream, &Messages::Pong).await?;
        Ok(())
    }

    pub async fn close(stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
        println!("debug: send close order");
        SFT::send(stream, &Messages::Close).await?;
        Ok(())
    }
    pub async fn open_session(
        stream: &mut tokio::net::TcpStream,
        files: &Vec<String>,
    ) -> anyhow::Result<Vec<u16>> {
        SFT::send(
            stream,
            &Messages::SessionInit {
                file_count: files.len(),
                filenames: files.clone(),
            },
        )
        .await?;
        match SFT::recv(stream).await? {
            Messages::SessionReady { ports } => Ok(ports),
            _ => Err(anyhow::anyhow!("failed to init transfert session")),
        }
    }

    pub async fn session_resp(
        stream: &mut tokio::net::TcpStream,
        ports: Vec<u16>,
    ) -> anyhow::Result<()> {
        SFT::send(stream, &Messages::SessionReady { ports: ports }).await?;
        Ok(())
    }
}

async fn stream_file_content(
    mut file: tokio::fs::File,
    stream: &mut tokio::net::TcpStream,
    size: u64,
    filename: &str,
    mp: &MultiProgress,
) -> anyhow::Result<()> {
    let pb = mp.add(ProgressBar::new(size));
    pb.set_style(
        ProgressStyle::with_template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_message(format_filename(filename, 15));

    let mut buf = vec![0u8; 64 * 2048];
    let mut total_send = 0u64;

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        total_send += n as u64;

        SFT::send(
            stream,
            &Messages::FileChunk {
                data: buf[..n].to_vec(),
            },
        )
        .await?;

        pb.set_position(total_send);

        // println!("Progress ({filename}): {total_send} => {size}");
    }
    pb.finish_with_message(format!("{} [done]", format_filename(filename, 15)));
    Ok(())
}

fn format_filename(filename: &str, width: usize) -> String {
    if filename.len() >= width {
        filename[..width].to_string()
    } else {
        format!("{:width$}", filename, width = width)
    }
}
