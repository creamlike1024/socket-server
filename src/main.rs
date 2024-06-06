use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> io::Result<()> {
    let address = std::env::args().nth(1).expect("no bind address given");
    let listener = TcpListener::bind(address)?;
    let clients = Arc::new(Mutex::new(Vec::new()));

    // 接受连接并处理它们
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer_addr = stream.peer_addr()?;
                let clients = clients.clone();
                println!("New connection: {}", peer_addr);

                // 将新客户端添加到列表中
                clients.lock().unwrap().push(stream.try_clone()?);

                thread::spawn(move || {
                    handle_client(stream, clients, peer_addr)
                        .unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
            Err(e) => {
                eprintln!("Failed to accept client: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    clients: Arc<Mutex<Vec<TcpStream>>>,
    peer_addr: SocketAddr,
) -> io::Result<()> {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by {}", peer_addr);
                break;
            }
            Ok(len) => {
                let msg = &buffer[..len];
                let clients_guard = clients.lock().unwrap();
                for mut client in clients_guard.iter() {
                    if client.peer_addr()? != peer_addr {
                        client.write_all(msg)?;
                    }
                }
            }
            Err(e) => {
                println!("Failed to read from {}: {}", peer_addr, e);
                break;
            }
        }
    }

    // 移除断开连接的客户端
    let mut clients_guard = clients.lock().unwrap();
    clients_guard.retain(|client| client.peer_addr().unwrap() != peer_addr);

    Ok(())
}
