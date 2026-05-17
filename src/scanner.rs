use tokio::net::{TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{timeout, Duration};
use tokio::task;

// TODO: 
// Create probe trait
// Create generic helper function for reading streams
// Create implementations for each probe type
// Update match statement to use new trait/impl

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
                Ok(Ok(_)) => {
                    println!("Port {} Is Open", port);
                    Some(port)
                },

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

pub async fn grab_banners(ip: &str, port: u16) -> std::io::Result<Option<String>> {
    let addr     = format!("{}:{}", ip, port);

    let mut stream = match timeout(
        Duration::from_secs(2),
        TcpStream::connect(&addr)
    ).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => return Err(e),
        Err(_) => return Ok(None),
    };

    match port {
        80 => {
            stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n").await?;
            stream.flush().await?;
        },
        443 => {
            stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n").await?;
            stream.flush().await?;
        }
        _ => {}
    }

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;

    if n == 0 {
        return Ok(None);
    }

    let banner = String::from_utf8_lossy(&buf[..n]).to_string();

    Ok(Some(banner))
}

