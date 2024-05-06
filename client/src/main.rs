/**
 * 主线程：
· 负责与服务器建立TCP连接。
· 创建一个通道（mpsc::channel）用于主线程与工作线程之间的通信。
· 监听用户的键盘输入。
· 当用户输入消息时，通过通道的发送端（tx）将消息发送给工作线程处理。
· 监听用户是否输入:quit命令或通道是否断开，以决定是否终止程序。

工作线程：
· 从通道的接收端（rx）接收主线程发送过来的消息。
· 处理与服务器的通信：
· 循环读取服务器发送来的消息，一旦有消息到达则打印。
· 将从通道接收到的消息转换为字节流，然后写入到与服务器的连接中发送出去。
· 控制循环和错误处理，如检测连接是否中断等。

主线程主要负责用户交互和消息的初步处理（即收集用户输入并传递），
而工作线程专注于与服务器的实际通信任务，包括接收服务器消息和向服务器发送由主线程转发过来的消息。
两者通过通道共享数据，实现任务的解耦和并行处理。
 */


use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;


const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect"); // 这里与本地服务器建立TCP连接
    client.set_nonblocking(true).expect("failed to initiate non-blocking");  // 这里是主线程

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {  // 这里只创建了一个工作线程 这利使用了move关键字 可以将外面定义的管道的所有权给工作线程
        /*
         * 这里设置缓冲区的目的：
         * 1. 直接操作内存缓冲区，比频繁操作io更加高效，我的理解是，当TCP数据报回来时候，直接将其中的数读到缓冲区利，方便操作，不用一直动不动就去其他网络层调用io
         * 2. 可以提前预留好符合网络标准的数据空间，不用每次在进行读取改写
         * 3. 在非阻塞I/O模型中，如果没有足够的数据立即可用，读取操作可能会返回错误（如WouldBlock）。使用缓冲区可以在数据逐步累积到足够处理时再进行处理，而不是立即报告错误，这提供了更细粒度的控制和错误处理策略。
         */
        let mut buff = vec![0;MSG_SIZE];  // 创建了一个包含32长度的数组 每个值都是0 将每个值设置为0的目的是：读入数据时，剩余未使用的缓冲区部分保持为零值

        match client.read_exact(&mut buff) {  // 这段代码 是工作线程用来从TCP 连接中读取数据 这个exact方法是正好可以读取到符合32限制的数据

            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();  // 这个msg表示接受到了 有效的新数组信息
                println!("message recv {:?}",msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                println!("message sent {:?}",msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("write a Message:");

    loop {
        let mut buff = String::new();  // 创建一个缓冲区

        io::stdin().read_line(&mut buff).expect("reading from stdin failed");  // 这里是主线程获得用户的输入，并且放入缓冲区

        let msg = buff.trim().to_string(); 

        if msg == ":quit" || tx.send(msg).is_err() {  // 只要如果接收端已经关闭，无法再接收更多的消息。返回err 就break
            break;
        }
    }
    println!("bye bye!");
}
