use std::{io, thread};
use std::io::Write;
use std::net::TcpStream;

fn main() -> io::Result<()> {
    thread::spawn(|| {
        loop {
            thread::sleep_ms(3000);
            //println!("\rHi world!");
        }
    });
    let mut buf = String::new();
    let mut conn = TcpStream::connect("127.0.0.1:8080")?;
    loop {
        io::stdin().read_line(&mut buf)?;
        print!("> ");
        io::stdout().flush();
        conn.write_all(buf.as_bytes())?;

        // message
        buf.clear();
    }
}
