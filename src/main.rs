use clap::{arg, Parser, Subcommand};
use dante_control_rs::{print_arc, print_chan, print_cmc, print_dbc, DanteDeviceManager};
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

    /// Monitors dante devices and prints device info every <print_interval> seconds.
    Monitor {
        #[arg(default_value_t = 2.0, short, long)]
        print_interval: f32,
    },

    /// Lists information about mDNS discovery on the "_netaudio-cmc._udp.local." address.
    PrintCMC {
        /// Seconds to wait for mDNS to resolve
        #[arg(default_value_t = 2.0, short, long)]
        time: f32,
    },

    /// Lists information about mDNS discovery on the "_netaudio-dbc._udp.local." address.
    PrintDBC {
        /// Seconds to wait for mDNS to resolve
        #[arg(default_value_t = 2.0, short, long)]
        time: f32,
    },

    /// Lists information about mDNS discovery on the "_netaudio-arc._udp.local." address.
    PrintARC {
        /// Seconds to wait for mDNS to resolve
        #[arg(default_value_t = 2.0, short, long)]
        time: f32,
    },

    /// Lists information about mDNS discovery on the "_netaudio-chan._udp.local." address.
    PrintCHAN {
        /// Seconds to wait for mDNS to resolve
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

            if !args.quiet {
                println!("Discovering Devices...");
            }

            sleep(Duration::from_secs_f32(*time));

            device_manager.stop_discovery();

            if !args.quiet {
                println!("Devices Found:");
            }

            for device_name in device_manager.get_device_names() {
                println!("{}", device_name);
            }
        }
        Some(Commands::Monitor { print_interval }) => {
            let device_manager = DanteDeviceManager::new();
            device_manager.start_discovery()?;

            if !args.quiet {
                println!("Starting monitoring");
            }

            loop {
                sleep(Duration::from_secs_f32(*print_interval));
                println!("==============================");
                for device_name in device_manager.get_device_names() {
                    println!("{}", device_name);
                }
            }
        }
        Some(Commands::PrintCMC { time }) => {
            print_cmc(Duration::from_secs_f32(*time));
        }
        Some(Commands::PrintDBC { time }) => {
            print_dbc(Duration::from_secs_f32(*time));
        }
        Some(Commands::PrintARC { time }) => {
            print_arc(Duration::from_secs_f32(*time));
        }
        Some(Commands::PrintCHAN { time }) => {
            print_chan(Duration::from_secs_f32(*time));
        }
        None => {
            println!("No command specified. Try \"dante-cli help\"");
        }
    }

    Ok(())
}
