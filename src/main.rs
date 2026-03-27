use std::sync::mpsc;

use anyhow::Result;
use clap::Parser;

use intercom::{IntercomConfig, run_intercom};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CommandLine {
    /// Local port for incoming audio
    #[arg(short, long, default_value = "0")]
    port: u16,

    /// Address of the destination computer
    #[arg(short, long)]
    destination: Option<String>,
}

fn main() -> Result<()> {
    let (interrupt_tx, interrupt_rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        let _ = interrupt_tx.send(());
    })?;

    let cli = CommandLine::parse();

    let mut config = IntercomConfig::with_port(cli.port);
    if let Some(dest) = cli.destination {
        config = config.send_to(dest);
    }

    let handle = run_intercom(config)?;
    println!("Intercom running! Press Ctrl+C to stop.");

    if let Ok(addr) = handle.get_receiver_address() {
        println!("Receiving on {addr}");
    }

    for addr in handle.get_destination_addresses().unwrap_or_default() {
        println!("Sending to {addr}");
    }

    interrupt_rx.recv()?;

    println!("\nShutting down gracefully...");
    drop(handle);

    Ok(())
}
