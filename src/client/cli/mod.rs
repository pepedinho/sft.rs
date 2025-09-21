use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Send { file: Vec<String>, dest: String },
}

#[derive(Debug, Clone)]
pub struct PackageInfos {
    pub file_path: Vec<String>,
    pub user: String,
    pub host: String,
}

impl PackageInfos {
    pub fn parse_command(cmd: Commands) -> anyhow::Result<PackageInfos> {
        match cmd {
            Commands::Send { file, dest } => {
                let s: Vec<&str> = dest.split('@').collect();
                if s.len() != 2 || s.iter().any(|v| v.is_empty()) {
                    Err(anyhow::anyhow!(
                        "incorrect host format: expected <user>@<host>"
                    ))
                } else {
                    Ok(PackageInfos {
                        file_path: file,
                        user: s[0].to_string(),
                        host: s[1].to_string(),
                    })
                }
            }
        }
    }
}
