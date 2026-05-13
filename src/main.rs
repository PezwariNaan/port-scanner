use clap::Parser;
mod scanner;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Target IP address
    #[arg(short, long)]
    ip: String,
    
    /// Ports to scan, seperated by commas
    #[arg(short, long, num_args = 1.., value_delimiter=',')]
    ports: Vec<u16>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Scanning IP: {}", args.ip);
    println!("Scanning Ports: {:?}", args.ports);

    let _ = scanner::scan_ports(&args.ip, args.ports).await;
}
