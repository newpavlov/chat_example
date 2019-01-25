use std::{io, thread, error};
use std::io::{Write, BufReader, BufRead};
use std::time::Duration;
use std::net::{TcpStream, SocketAddr};
use std::sync::mpsc;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
    /// User nickname
    #[structopt(short = "u", long = "user")]
    user: String,
    /// Server socket address
    #[structopt(short = "a", long = "addr", default_value = "127.0.0.1:8080")]
    addr: SocketAddr,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = Opt::from_args();

    let mut stream = TcpStream::connect(opt.addr)?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;

    stream.write_all(opt.user.as_bytes())?;
    stream.write_all(b"\n")?;

    let mut stream = BufReader::new(stream);
    let mut line = String::new();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || -> io::Result<()> {
        loop {
            let mut buf = String::new();

            print!("> ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut buf)?;

            tx.send(buf)
                .map_err(|err| io::Error::new(
                    io::ErrorKind::Other,
                    err
                ))?;
        }
    });

    loop {
        let res = stream.read_line(&mut line);
        match res {
            Ok(0) => {
                println!("\rServer connection closed.");
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
        line.clear();
    }
}
