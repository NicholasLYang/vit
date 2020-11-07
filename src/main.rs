#[macro_use]
extern crate nom;

pub mod parser;

fn main() {
    let input = "   TITLE:";
    match parser::length_value(input.as_bytes()) {
        Ok(res) => {
            println!("{:?}", res);
        },
        Err(err) => {
            println!("ERROR: {:?}", err);
        }
    }

}
