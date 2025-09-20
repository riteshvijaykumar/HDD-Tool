mod device;
mod wipe;
mod verify;
mod report;
mod util;
mod sanitization;
mod web_api;

use clap::{Parser, Subcommand};
use anyhow::Result;
use device::Device;
use sanitization::SafeWipeController;
use wipe::SanitizationMethod;

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
    /// Start the web GUI server
    Gui {
        /// Port to run the web server on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Scan and list all available storage devices with capabilities
    Scan,
    /// List devices (legacy command)
    List,
    /// Get recommendations for device sanitization
    Recommend,
    /// Sanitize specific devices with chosen method
    Sanitize {
        /// Sanitization method: clear, purge, or destroy
        #[arg(short, long, default_value = "clear")]
        method: String,
        /// Device names to sanitize (comma-separated)
        #[arg(short, long)]
        devices: Option<String>,
        /// Sanitize all non-system devices
        #[arg(long)]
        all: bool,
    },
    /// Legacy wipe command (deprecated)
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
    println!("🛡️ SafeWipe Engine - NIST SP 800-88 Compliant Data Sanitization");
    println!("================================================================");
    println!();

    let cli = Cli::parse();

    match cli.command {
        Commands::Gui { port } => {
            println!("🚀 Starting SafeWipe Web GUI...");
            web_api::start_web_server(port).await?;
        }

        Commands::Scan => {
            let mut controller = SafeWipeController::new()
                .with_progress_callback(|progress| {
                    println!("📊 Progress: {:.1}% ({} bytes processed)",
                        (progress.bytes_processed as f64 / progress.total_bytes as f64) * 100.0,
                        progress.bytes_processed);
                });

            let devices = controller.scan_drives().await?;

            if devices.is_empty() {
                println!("❌ No storage devices found.");
            } else {
                println!("✅ Scan completed. Found {} device(s).", devices.len());
            }
        }

        Commands::List => {
            println!("📋 Listing devices using legacy detection...");
            let devices = device::list_devices()?;
            println!("Available devices:");
            for dev in devices {
                println!("  {:?}", dev);
            }
        }

        Commands::Recommend => {
            let mut controller = SafeWipeController::new();
            let devices = controller.scan_drives().await?;
            let recommendations = controller.get_recommendations(&devices);

            println!("💡 Sanitization Recommendations:");
            println!("================================");
            for (device_name, recommendation) in recommendations {
                println!("📦 {}: {}", device_name, recommendation);
            }
        }

        Commands::Sanitize { method, devices, all } => {
            let sanitization_method = match method.to_lowercase().as_str() {
                "clear" => SanitizationMethod::Clear,
                "purge" => SanitizationMethod::Purge,
                "destroy" => SanitizationMethod::Destroy,
                _ => {
                    println!("❌ Invalid method. Use: clear, purge, or destroy");
                    return Ok(());
                }
            };

            let mut controller = SafeWipeController::new()
                .with_progress_callback(|progress| {
                    println!("📊 Progress: Pass {}/{} - {:.1}% complete",
                        progress.current_pass,
                        progress.total_passes,
                        (progress.bytes_processed as f64 / progress.total_bytes as f64) * 100.0);
                });

            let all_devices = controller.scan_drives().await?;

            let selected_devices: Vec<Device> = if all {
                all_devices.into_iter()
                    .filter(|d| !d.is_system_drive)
                    .collect()
            } else if let Some(device_list) = devices {
                let device_names: Vec<&str> = device_list.split(',').collect();
                all_devices.into_iter()
                    .filter(|d| device_names.contains(&d.name.as_str()))
                    .collect()
            } else {
                println!("❌ Please specify devices with --devices or use --all flag");
                return Ok(());
            };

            if selected_devices.is_empty() {
                println!("❌ No valid devices selected for sanitization");
                return Ok(());
            }

            // Create and review sanitization plan
            let plan = controller.create_sanitization_plan(selected_devices, sanitization_method)?;

            println!("📋 Sanitization Plan:");
            println!("====================");
            println!("Method: {:?}", plan.method);
            println!("Devices: {}", plan.devices.len());
            println!("Estimated Duration: {:?}", plan.estimated_duration);
            println!();

            if !plan.safety_warnings.is_empty() {
                println!("⚠️ Safety Warnings:");
                for warning in &plan.safety_warnings {
                    println!("  {}", warning);
                }
                println!();
            }

            // Confirm before proceeding
            println!("❓ Do you want to proceed? (yes/no):");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "yes" {
                println!("❌ Operation cancelled.");
                return Ok(());
            }

            // Execute the plan
            let report = controller.execute_plan(plan).await?;

            println!();
            println!("📊 Final Report:");
            println!("================");
            println!("{}", report.summary);

            for result in &report.results {
                println!("📦 {}: {:?}", result.device.name, result.status);
                if !result.patterns_used.is_empty() {
                    println!("   Patterns used: {}", result.patterns_used.join(", "));
                }
                if let Some(duration) = result.duration {
                    println!("   Duration: {:?}", duration);
                }
            }
        }

        Commands::Wipe { device, method } => {
            println!("⚠️ Using legacy wipe command. Consider using 'sanitize' for full functionality.");
            wipe::wipe_device(&device, &method).await?;
        }

        Commands::Report { device } => {
            println!("📊 Generating sanitization report for device: {}", device);
            // This would integrate with the report module
            println!("Report generation not yet implemented in this demo.");
        }
    }

    Ok(())
}
