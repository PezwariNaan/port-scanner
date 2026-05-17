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

    println!("Scanning IP: {}", args.ip);
    println!("Scanning Ports: {:?}", args.ports);

    let open_ports = scanner::scan_ports(&args.ip, args.ports).await;
    let banners = scanner::grab_banners(&args.ip, open_ports[0]).await;

    banners.iter().for_each(|banner| println!("{:?}", banner));
}
