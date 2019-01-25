use std::{io, thread, error};
use std::io::{Write, BufReader, BufRead};
use std::time::Duration;
use std::net::TcpStream;
use std::sync::mpsc;

fn handle_connection(rx: mpsc::Receiver<String>) -> Result<(), Box<dyn error::Error>> {
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    let mut stream = BufReader::new(stream);
    let mut line = String::new();

    loop {
        let res = stream.read_line(&mut line);
        match res {
            Ok(0) => {
                println!("server connection closed");
                return Ok(());
            },
            Ok(_) => (),
            Err(err) => {
                if err.kind() == io::ErrorKind::WouldBlock {
                    let msg = match rx.try_recv() {
                        Ok(msg) => msg,
                        Err(mpsc::TryRecvError::Empty) => continue,
                        Err(err) => return Err(Box::new(err)),
                    };
                    stream.get_mut().write_all(msg.as_bytes())?;
                    continue;
                } else {
                    return Err(Box::new(err));
                }
            },
        };
        print!("\r{}", line);
        print!("> ");
        io::stdout().flush()?;
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut buf = String::new();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Err(err) = handle_connection(rx) {
            eprintln!("Error: {:?}", err);
        }
    });
    loop {
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut buf)?;

        tx.send(buf.clone())?;

        buf.clear();
    }
}
