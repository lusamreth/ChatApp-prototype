mod backend;
mod domain;
mod http;
mod pipe;
fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let sys = actix::System::new("Server-one");
    if let Err(e) = http::build() {
        eprintln!("Error server ${:#?}", e);
    }
    sys.run()
}
