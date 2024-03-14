use arithmetic::arithmetic;

fn main() {
    println!("{:?}", arithmetic(""));
    println!("{:?}", arithmetic("-10%"));
    println!("{:?}", arithmetic("-5 * (4 - 1) + 20 / 4"));
}
