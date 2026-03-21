use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod elm327_sim;

#[derive(Parser)]
#[command(name = "elm327-simulator")]
#[command(about = "Simulated ELM327 v1.5 device for testing")]
struct Cli {
    /// Print device path and exit (for scripting)
    #[arg(long)]
    print_path: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    // Init logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // Create PTY pair using nix directly.
    // TODO: switch to elm327-core's PtyPair once it's available
    let pty = nix::pty::openpty(None, None).expect("Failed to create PTY pair");

    let device_path =
        nix::unistd::ttyname(&pty.slave).expect("Failed to get PTY device path");

    // Always print the path so scripts can capture it
    println!("{}", device_path.display());

    if cli.print_path {
        return;
    }

    log::info!("ELM327 simulator running on {}", device_path.display());
    log::info!("Connect your bridge to this device path");

    // Setup shutdown handler
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    ctrlc::set_handler(move || {
        shutdown_clone.store(true, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl+C handler");

    // The simulator reads/writes on the master (controller) end.
    // External clients connect to the slave (device_path).
    use std::os::fd::{FromRawFd, IntoRawFd};
    let master_fd = pty.master.into_raw_fd();
    let mut file = unsafe { std::fs::File::from_raw_fd(master_fd) };

    // Keep slave fd alive so the device path stays valid
    let _slave = pty.slave;

    let mut sim = elm327_sim::Elm327Simulator::new();
    if let Err(e) = sim.run(&mut file, &shutdown) {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            log::info!("Client disconnected");
        } else {
            log::error!("Simulator error: {}", e);
        }
    }

    log::info!("Simulator stopped");
}
