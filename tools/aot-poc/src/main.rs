fn main() {
    let status = match neoclr_aot_poc::run(std::env::args_os().skip(1).collect()) {
        Ok(status) => status,
        Err(error) => {
            eprintln!("aot-poc: {error}");
            1
        }
    };
    if status != 0 {
        std::process::exit(status);
    }
}
