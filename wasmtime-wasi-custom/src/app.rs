fn main() {}

#[no_mangle]
pub extern fn hello_world() {
    println!("Hello, world!");
    eprintln!("Hello, stderr!");
}