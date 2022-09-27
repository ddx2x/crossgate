use std::io::{self, BufRead, Write};

use condition::parse;

fn main() {
    let stdin = io::stdin();
    loop {
        print!(">>> ");
        io::stdout().flush().ok();
        match stdin.lock().lines().next() {
            Some(Ok(ref l)) => match parse(l) {
                Ok(rs) => println!("{:#?}", rs),
                Err(e) => eprintln!("{:?}", e),
            },
            _ => break,
        }
    }
}
