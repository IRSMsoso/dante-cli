use ascii::AsAsciiStr;
use clap::{arg, Parser, Subcommand};
use dante_control_rs::{print_arc, print_chan, print_cmc, print_dbc, DanteDeviceManager, DanteVersion};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::net::Ipv4Addr;
use std::str::FromStr;
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
        /// Dante version to use. Possible values are "4.4.1.3" and "4.2.1.3"
        version: String,

        /// Name of the dante device to transmit the new subscription
        transmitter_name: String,

        /// Channel id of the dante device to transmit the new subscription
        transmitter_channel_name: String,

        /// Ip of the dante device to receive the new subscription
        receiver_ip_string: String,

        /// Channel id of the dante device to receive the new subscription
        receiver_channel_index: u16,
    },

    /// Make subscription
    ClearSubscription {
        /// Dante version to use. Possible values are "4.4.1.3" and "4.2.1.3"
        version: String,

        /// Ip of the dante device to receive the new subscription
        receiver_ip_string: String,

        /// Channel id of the dante device to receive the new subscription
        receiver_channel_index: u16,
    },

    /// Make a series of subscriptions as specified in plaintext from a file, where each line is another subscription and looks like this: TransmitterChannelName@TransmitterDeviceName:ReceiverChannelIndex@ReceiverIp. Note the receiver using an index instead of a channel name. Clear the subscription by only providing the receiver ip and channel index: receiver_index@receiver_ip
    MakeSubscriptionsFromFile {
        /// Path of file to read from.
        file_path: String,
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

#[derive(thiserror::Error, Debug)]
pub enum SubscriptionError {
    #[error("Couldn't parse version argument into a valid Dante Version.")]
    VersionParse,
}


#[derive(thiserror::Error, Debug)]
pub enum ParsingError {
    #[error("Could not properly detect : between the transmitting and receiving devices")]
    TxRxDelimiter,
    #[error("Could not properly detect @ between the transmitting channel and device name")]
    TxDelimiter,
    #[error("Could not properly detect @ between the receiving channel index and device name")]
    RxDelimiter,
    #[error("Could not properly detect | between the version and tx/rx devices")]
    VersionDelimiter,
    #[error("Could not parse the receiving channel index into an integer")]
    RxChanIndexParse,
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
        Some(Commands::Control(control_command)) => match control_command {
            ControlCommands::MakeSubscription {
                version,
                transmitter_name,
                transmitter_channel_name,
                receiver_ip_string,
                receiver_channel_index,
            } => {
                let version = DanteVersion::from_string(version).ok_or(SubscriptionError::VersionParse)?;


                let receiver_ip = Ipv4Addr::from_str(receiver_ip_string)?;
                let transmitter_name_ascii = transmitter_name.as_ascii_str()?;
                let transmitter_channel_name_ascii = transmitter_channel_name.as_ascii_str()?;

                let mut device_manager = DanteDeviceManager::new();

                device_manager.make_subscription(
                    version,
                    &receiver_ip,
                    *receiver_channel_index,
                    transmitter_name_ascii,
                    transmitter_channel_name_ascii,
                )?;
            }
            ControlCommands::MakeSubscriptionsFromFile { file_path} => {
                let mut device_manager = DanteDeviceManager::new();

                let file = File::open(file_path)?;
                let lines = io::BufReader::new(file).lines();
                for line in lines.flatten() {
                    let (version_string, command_string) = line.split_once('|').ok_or(ParsingError::VersionDelimiter)?;
                    let version: DanteVersion = DanteVersion::from_string(version_string).ok_or(SubscriptionError::VersionParse)?;

                    if command_string.contains(':') {
                        let (tx, rx) = command_string.split_once(':').ok_or(ParsingError::TxRxDelimiter)?;
                        let (tx_chan, tx_device) =
                            tx.split_once('@').ok_or(ParsingError::TxDelimiter)?;
                        let (rx_chan, rx_ip_string) =
                            rx.split_once('@').ok_or(ParsingError::RxDelimiter)?;
                        let rx_chan_index: u16 = match rx_chan.parse() {
                            Ok(chan_index) => Ok(chan_index),
                            Err(_) => Err(ParsingError::RxChanIndexParse),
                        }?;

                        let receiver_ip = Ipv4Addr::from_str(rx_ip_string)?;
                        let transmitter_name_ascii = tx_device.as_ascii_str()?;
                        let transmitter_channel_name_ascii = tx_chan.as_ascii_str()?;

                        device_manager.make_subscription(
                            version,
                            &receiver_ip,
                            rx_chan_index,
                            transmitter_name_ascii,
                            transmitter_channel_name_ascii,
                        )?;
                    } else {
                        let (rx_chan, rx_ip_string) =
                            command_string.split_once('@').ok_or(ParsingError::RxDelimiter)?;
                        let rx_chan_index: u16 = match rx_chan.parse() {
                            Ok(chan_index) => Ok(chan_index),
                            Err(_) => Err(ParsingError::RxChanIndexParse),
                        }?;

                        let receiver_ip = Ipv4Addr::from_str(rx_ip_string)?;

                        device_manager.clear_subscription(version, &receiver_ip, rx_chan_index)?;
                    }
                }
            }
            ControlCommands::ClearSubscription {
                version,
                receiver_ip_string,
                receiver_channel_index,
            } => {
                let version = DanteVersion::from_string(version).ok_or(SubscriptionError::VersionParse)?;

                let receiver_ip = Ipv4Addr::from_str(receiver_ip_string)?;

                let mut device_manager = DanteDeviceManager::new();

                device_manager.clear_subscription(version, &receiver_ip, *receiver_channel_index)?;
            }
        },
        None => {
            println!("No command specified. Try \"dante-cli help\"");
        }
    }

    Ok(())
}
