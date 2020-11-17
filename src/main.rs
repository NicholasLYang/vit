extern crate nom;

pub mod parser;

fn main() {
    let input = "   TITLE: ZAMA
001 tape editchannels edittype clipstarttime clipendtime timelineposition timelinepositionend";
    let res = parser::parse_edl_file(input.as_bytes());
    println!("{:?}", res);
}
