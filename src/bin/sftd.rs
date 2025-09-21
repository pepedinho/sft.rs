use sft::server::Listener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Listener::listen_on("0.0.0.0:8000").await?;

    Ok(())
}
