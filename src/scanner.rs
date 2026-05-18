use tokio::net::{TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncRead};
use tokio::time::{timeout, Duration};
use tokio::task;
use async_trait::async_trait;
use std::net::Ipv4Addr;

// TODO: 
// Create implementations for each probe type

#[async_trait]
pub trait ServiceProbe {
    async fn probe(
        &self,
        stream: &mut TcpStream,
    ) -> std::io::Result<Option<String>>;
}

pub struct HttpProbe;
pub struct GenericProbe;

#[async_trait]
impl ServiceProbe for HttpProbe {
    async fn probe(
        &self,
        stream: &mut TcpStream,
    ) -> std::io::Result<Option<String>> {
        stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n").await?;
        stream.flush().await?;

        read_banner(stream).await
    }
}

#[async_trait]
impl ServiceProbe for GenericProbe {
    async fn probe(
        &self,
        stream: &mut TcpStream,
    ) -> std::io::Result<Option<String>> {
        read_banner(stream).await
    }
}

async fn scan_ports(ip: String, ports: Vec<u16>) -> Vec<u16> {
    let mut handles    = Vec::new();

    for port in ports {
        let addr = format!("{}:{}", ip, port);

        let handle = task::spawn( async move {
            let result = timeout(
                Duration::from_millis(500),
                TcpStream::connect(&addr),
            ).await;

            match result {
                Ok(Ok(mut stream)) => {
                    match grab_banner(&mut stream, port).await {
                        Ok(Some(banner)) => {
                            println!(
                                "Ip: {}\nPort: {} Open\nBanner: {}",
                                addr.split(":").collect::<Vec<_>>()[0],
                                port,
                                banner.trim()
                            );
                        }
                        
                        Ok(None)=> {
                            println!(
                                "Port {} No Banner",
                                port
                            );
                        }

                        Err(e) => {
                            println!(
                                "Port {} Probe Error : {}",
                                port,
                                e
                            );
                        }
                    }
                    Some(port)
                }
                _ => None,
            }
        });
        handles.push(handle);
    }
    
    let mut open_ports = Vec::new();

    for handle in handles {
        if let Ok(Some(port)) = handle.await {
            open_ports.push(port);
        }
    }

    open_ports
}

async fn grab_banner(
    stream: &mut TcpStream, 
    port: u16
) -> std::io::Result<Option<String>> {
    let probe = select_probe(port);

    probe.probe(stream).await
}

pub async fn read_banner<S>(
    stream: &mut S,
) -> std::io::Result<Option<String>>
where 
    S: AsyncRead + Unpin,
{
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;

    if n == 0 {
        return Ok(None);
    }

    Ok(Some(
        String::from_utf8_lossy(&buf[..n]).to_string()
    ))
}

fn select_probe(port: u16) -> Box<dyn ServiceProbe + Send + Sync> {
    match port {
        80 => Box::new(HttpProbe),
        _  => Box::new(GenericProbe),
    }
}

fn subnet_mask(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        (!0u32) << (32 - prefix)
    }
}

fn parse_ip(ip_input: &str) -> Result<Vec<Ipv4Addr>, String> {
    if !ip_input.contains('/') {
        let ip: Ipv4Addr = ip_input
            .parse()
            .map_err(|_| "Invalid IP Address".to_string())?;

        return Ok(vec![ip]);
    }

    let mut ip_range = Vec::new();

    let (ip_str, prefix_str) = ip_input
        .split_once('/')
        .ok_or("Missing CIDR Prefix")?;

    let ip:Ipv4Addr = ip_str
        .parse()
        .map_err(|_| "Invalid IP Address".to_string())?;

    let prefix: u8 = prefix_str
        .parse()
        .map_err(|_| "Invalid Prefix".to_string())?;

    if prefix > 32 {
        return Err("Prefix must be <= 32".into());
    }

    let network = u32::from(ip) & subnet_mask(prefix);
    let broadcast_address = network | !subnet_mask(prefix);

    for host in (network + 1)..broadcast_address {
        ip_range.push(Ipv4Addr::from(host))
    }

    Ok(ip_range)
}

pub async fn scan_ips(ip_input: &str, ports: Vec<u16>) -> () {
    let ips = parse_ip(ip_input).unwrap();

    let mut handles = Vec::new();

    for ip in ips {
        let ip_addr = ip.to_string();
        let ports_clone = ports.clone();

        let handle = task::spawn(async move {
            scan_ports(ip_addr, ports_clone).await
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}

