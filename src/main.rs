use clap::{Parser, Subcommand};
use dante_control_rs::DanteDeviceManager;
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about = "Command line tool for interacting with dante devices on the local network", long_about = None)]
struct Args {
    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Lists the available dante devices on the local network
    ListDevices {
        /// Seconds to wait for mDNS to resolve before printing discovered devices
        #[arg(default_value_t = 2.0, short, long)]
        time: f32,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    stderrlog::new()
        .module(module_path!())
        .module("dante_control_rs")
        .quiet(args.quiet)
        .verbosity(args.verbose as usize)
        .init()
        .expect("Failed to initialize stderrlog");

    match &args.command {
        Some(Commands::ListDevices { time }) => {
            let device_manager = DanteDeviceManager::new();
            device_manager.start_discovery()?;

            println!("Discovering Devices...");

            sleep(Duration::from_secs_f32(*time));

            device_manager.stop_discovery();

            println!("Devices Found:");
            for device_name in device_manager.get_device_names() {
                println!("{}", device_name);
            }
        }
        None => {
            println!("No command specified. Try \"dante-cli help\"");
        }
    }

    Ok(())
}
