use sft::server::Listener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Listener::listen_on("127.0.0.1:8000").await?;

    Ok(())
}
