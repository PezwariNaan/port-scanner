use clap::Parser;
mod scanner;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Target IP address
    #[arg(short, long)]
    ip: String,
    
    /// Ports to scan, seperated by commas
    #[arg(short, long, num_args = 1.., value_delimiter = ',', required = true)]
    ports: Vec<u16>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let _open_ports = scanner::scan_ips(&args.ip, args.ports).await;
}
