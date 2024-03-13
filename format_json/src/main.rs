use std::fs;

use format_json::Formatter;

fn main() {
    let mut fmt = Formatter::default();
    let json = fs::read_to_string("./format_json/data.json").unwrap();
    println!("{}", fmt.format(&json));
    // println!("{}", fmt.format_raw(&json));
}
