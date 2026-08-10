fn main() {
    if let Err(err) = cashtrack::run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
