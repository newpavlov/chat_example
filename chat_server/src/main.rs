use std::net::{TcpListener, TcpStream};
use std::{io, thread};
use std::io::{BufReader, BufRead};
use std::time::Duration;
use std::sync::{Mutex, Arc};

type History = Arc<Mutex<Vec<String>>>;

fn handle_client(stream: io::Result<TcpStream>, history: History) -> io::Result<()> {
    let stream = stream?;
    let addr = stream.peer_addr();
    println!("got connection: {:?}", addr);
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    let stream = BufReader::new(stream);
    let mut pos = 0;

    for line in stream.lines() {
        let line = match line {
            Ok(l) => l,
            Err(err) => {
                if err.kind() == io::ErrorKind::WouldBlock {
                    continue;
                } else {
                    return Err(err);
                }
            },
        };
        let mut history_lock = history.lock().unwrap();
        println!("client wrote: [{}] {:?}",
            history_lock.len(), line);
        history_lock.push(line);
    }
    println!("connection closed: {:?}", addr);
    Ok(())
}

fn main()  -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let history = Arc::new(Mutex::new(vec![]));
    for stream in listener.incoming() {
        let history = history.clone();
        thread::spawn(move || {
            if let Err(err) = handle_client(stream, history) {
                eprintln!("Error detected: {:?}", err);
            }
        });
    }
    Ok(())
}
