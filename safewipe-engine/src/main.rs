mod device;
mod wipe;
mod verify;
mod report;
mod util;

use clap::{Parser, Subcommand};
use anyhow::Result;

/// SafeWipe CLI - NIST SP 800-88 Compliant Data Sanitization
#[derive(Parser)]
#[command(name = "safewipe")]
#[command(about = "Secure data sanitization tool (NIST SP 800-88 compliant)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available storage devices
    List,
    /// Wipe a device with a specified method
    Wipe {
        #[arg(short, long)]
        device: String,
        #[arg(short, long, default_value = "clear")]
        method: String,
    },
    /// Generate sanitization report
    Report {
        #[arg(short, long)]
        device: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let devices = device::list_devices()?;
            println!("Available devices:");
            for dev in devices {
                println!("{:?}", dev);
            }
            Ok(())
        }
        Commands::Wipe { device, method } => {
            wipe::wipe_device(&device, &method).await?;
            Ok(())
        }
        Commands::Report { device } => {
            report::generate_report(&device)?;
            Ok(())
        }
    }
}
