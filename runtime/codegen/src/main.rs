fn main() {
    let status = match neoclr_codegen::run(std::env::args_os().skip(1).collect()) {
        Ok(status) => status,
        Err(error) => {
            eprintln!("neoclr-codegen: {error}");
            1
        }
    };
    if status != 0 {
        std::process::exit(status);
    }
}
