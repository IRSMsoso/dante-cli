use ascii::AsAsciiStr;
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
        #[arg(default_value_t = 5.0, short, long)]
        time: f32,

        /// Print detailed info instead of just device names.
        #[arg(short, long)]
        detailed: bool,
    },

    /// Monitors dante devices and prints device info every <print_interval> seconds.
    Monitor {
        /// Interval to print values to stdout
        #[arg(default_value_t = 2.0, short, long)]
        print_interval: f32,

        /// Print detailed info instead of just device names.
        #[arg(short, long)]
        detailed: bool,
    },

    /// Command for controlling dante devices.
    #[command(subcommand)]
    Control(ControlCommands),

    /// Debug commands (mostly for mDNS).
    #[command(subcommand)]
    Debug(DebugCommands),
}

#[derive(Subcommand, Debug)]
enum ControlCommands {
    /// Make subscription
    MakeSubscription {
        /// Name of the dante device to transmit the new subscription
        transmitter_name: String,

        /// Channel id of the dante device to transmit the new subscription
        transmitter_channel_id: u16,

        /// Name of the dante device to receive the new subscription
        receiver_name: String,

        /// Channel id of the dante device to receive the new subscription
        receiver_channel_id: u16,

        /// Seconds to wait for mDNS to resolve before printing discovered devices
        #[arg(default_value_t = 5.0, short, long)]
        time: f32,
    },
}

#[derive(Subcommand, Debug)]
enum DebugCommands {
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
        Some(Commands::ListDevices { time, detailed }) => {
            let device_manager = DanteDeviceManager::new();
            device_manager.start_discovery()?;

            if !args.quiet {
                println!("Discovering Devices...");
            }

            sleep(Duration::from_secs_f32(*time));

            device_manager.stop_discovery();

            if !args.quiet {
                println!("Devices Found:\n");
            }

            if !*detailed {
                for device_name in device_manager.get_device_names() {
                    println!("{}", device_name);
                }
            } else {
                for device_info in device_manager.get_device_descriptions() {
                    println!("{}", device_info);
                    println!("---------------------------------");
                }
            }
        }
        Some(Commands::Monitor {
            print_interval,
            detailed,
        }) => {
            let device_manager = DanteDeviceManager::new();
            device_manager.start_discovery()?;

            if !args.quiet {
                println!("Starting monitoring");
            }

            loop {
                sleep(Duration::from_secs_f32(*print_interval));
                println!("=================================");
                if !*detailed {
                    for device_name in device_manager.get_device_names() {
                        println!("{}", device_name);
                    }
                } else {
                    for device_info in device_manager.get_device_descriptions() {
                        println!("{}", device_info);
                        println!("---------------------------------");
                    }
                }
            }
        }
        Some(Commands::Debug(debug_command)) => match debug_command {
            DebugCommands::PrintCMC { time } => {
                print_cmc(Duration::from_secs_f32(*time));
            }
            DebugCommands::PrintDBC { time } => {
                print_dbc(Duration::from_secs_f32(*time));
            }
            DebugCommands::PrintARC { time } => {
                print_arc(Duration::from_secs_f32(*time));
            }
            DebugCommands::PrintCHAN { time } => {
                print_chan(Duration::from_secs_f32(*time));
            }
        },
        None => {
            println!("No command specified. Try \"dante-cli help\"");
        }
        Some(Commands::Control(control_command)) => match control_command {
            ControlCommands::MakeSubscription {
                transmitter_name,
                transmitter_channel_id,
                receiver_name,
                receiver_channel_id,
                time,
            } => {
                let receiver_name_ascii = receiver_name.as_ascii_str()?;
                let transmitter_name_ascii = transmitter_name.as_ascii_str()?;

                let mut device_manager = DanteDeviceManager::new();
                device_manager.start_discovery()?;

                if !args.quiet {
                    println!("Discovering Devices...");
                }

                sleep(Duration::from_secs_f32(*time));

                device_manager.stop_discovery();

                device_manager.make_subscription(
                    receiver_name_ascii,
                    *receiver_channel_id,
                    transmitter_name_ascii,
                    *transmitter_channel_id,
                )?;
            }
        },
    }

    Ok(())
}
