use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tokio::task;

pub async fn scan_ports(ip: &str, ports: Vec<u16>) -> () {
    let mut handles = Vec::new();

    for port in ports {
        let addr = format!("{}:{}", ip, port);

        let handle = task::spawn( async move {
            let result = timeout(
                Duration::from_millis(500),
                TcpStream::connect(&addr),
            ).await;

            match result {
                Ok(Ok(_)) => println!("Port {} Is Open", port),
                Ok(Err(_)) => {},
                Err(_) => {}
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}

