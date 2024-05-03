use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener; // 创建服务器
use std::sync::mpsc; // 多线程之间的通信
use std::thread;  


const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;
fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind"); // 创建服务器并且绑定端口
    
    server.set_nonblocking(true).expect("failed to initialize non-bocking");  // 给服务器设定非阻塞

    let mut clients = vec![]; // 创建多个客户端连接服务器 因为聊天不会只有一个客户端登入服务器
    let (tx, rx) = mpsc::channel::<String>(); // 设定通道呢内通信的信息是String

    loop { // 这里是主线程干的事 在持续的监听客户端的连接
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        println!("{}: {:?}",addr, msg);
                        tx.send(msg).expect("failed to send msg to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("closing connection with: {}", addr);
                        break;
                    }
                }
            });
        }
    }
}
