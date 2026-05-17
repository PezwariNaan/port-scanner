use tokio::net::{TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncRead};
use tokio::time::{timeout, Duration};
use tokio::task;
use async_trait::async_trait;

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
        22 => Box::new(GenericProbe),
        _  => Box::new(GenericProbe),
    }
}

pub async fn scan_ports(ip: &str, ports: Vec<u16>) -> Vec<u16> {
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
                    println!("Port {} Is Open", port);
                    match grab_banner(&mut stream, port).await {
                        Ok(Some(banner)) => {
                            println!(
                                "Port {} Banner: {}",
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

