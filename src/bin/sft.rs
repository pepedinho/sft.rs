use clap::Parser;
use sft::client::cli::{Cli, PackageInfos};

fn main() -> anyhow::Result<()> {
    let pckginfos = PackageInfos::parse_command(Cli::parse().command)?;
    println!("{:#?}", pckginfos);
    Ok(())
}
