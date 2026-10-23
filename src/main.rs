fn main() {
    if let Err(error) = vertex_dtl::runtime::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
