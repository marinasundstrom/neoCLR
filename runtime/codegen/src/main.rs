fn main() {
    // Debug codegen frames can exceed the Windows executable's default stack.
    // Match the existing lazy compiler worker budget; release entry is unchanged.
    #[cfg(all(windows, debug_assertions))]
    {
        let worker = std::thread::Builder::new()
            .name("neoclr-codegen".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(run_main)
            .expect("could not start compiler worker");
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
    }
    #[cfg(not(all(windows, debug_assertions)))]
    run_main();
}

fn run_main() {
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
