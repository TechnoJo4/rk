use std::io::{self, Write};

mod value;
mod print;
mod parse;
mod bc;
mod vm;

fn main() {
    loop {
        let mut line = String::new();
        print!(">");
        io::stdout().flush().unwrap();

        if io::stdin().read_line(&mut line).is_err() { break }

        if line.is_empty() { break }

        let parsed = parse::parse(line.trim_end().to_owned());
        println!("{:?}", parsed);

        if let Ok(node) = parsed {
            let fun = bc::compile(node);
            println!("{:?}", fun);
        }
    }
}
