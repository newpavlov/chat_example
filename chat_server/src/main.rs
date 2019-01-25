use std::net::{TcpListener, TcpStream};
use std::{io, thread};
use std::io::{BufReader, BufRead, Write};
use std::time::Duration;
use std::sync::{Mutex, Arc};

type History = Arc<Mutex<Vec<String>>>;

fn send_history(history: &[String], pos: &mut usize, stream: &mut TcpStream) -> io::Result<()> {
    if *pos < history.len() {
        for msg in &history[*pos..] {
            stream.write_all(msg.as_bytes())?;
        }
        *pos = history.len();
    }
    Ok(())
}

fn handle_client(stream: io::Result<TcpStream>, history: History) -> io::Result<()> {
    let stream = stream?;
    let addr = stream.peer_addr();
    println!("got connection: {:?}", addr);
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    let mut stream = BufReader::new(stream);
    let mut pos = {
        let n = history.lock().unwrap().len();
        if n < 10 {
            0
        } else {
            n - 10
        }
    };

    loop {
        let mut line = String::new();
        let res = stream.read_line(&mut line);
        match res {
            Ok(0) => {
                println!("connection closed: {:?}", addr);
                return Ok(());
            },
            Ok(_) => (),
            Err(err) => {
                if err.kind() == io::ErrorKind::WouldBlock {
                    let history_lock = history.lock().unwrap();
                    send_history(&history_lock, &mut pos, stream.get_mut())?;
                    continue;
                } else {
                    return Err(err);
                }
            },
        };

        let mut history_lock = history.lock().unwrap();
        send_history(&history_lock, &mut pos, stream.get_mut())?;
        println!("client wrote: [{}] {:?}",
            history_lock.len(), line);
        history_lock.push(line);
        pos += 1;
    }
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
