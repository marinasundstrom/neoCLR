use std::process::Command;

#[test]
fn cli_gc_statistics_are_opt_in_and_do_not_change_program_stdout() {
    let run = |stats: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr"));
        command.args(["run", "examples/features.neoil"]);
        if stats {
            command.arg("--gc-stats");
        }
        command.output().unwrap()
    };
    let ordinary = run(false);
    let diagnosed = run(true);
    assert!(ordinary.status.success());
    assert!(
        diagnosed.status.success(),
        "{}",
        String::from_utf8_lossy(&diagnosed.stderr)
    );
    assert_eq!(ordinary.stdout, diagnosed.stdout);
    assert!(ordinary.stderr.is_empty());
    assert_eq!(
        String::from_utf8(diagnosed.stderr).unwrap(),
        "GC: allocated=1 live=0 peak=1 collections=1 reclaimed=1\n"
    );
}

#[test]
fn cli_rejects_gc_statistics_for_analysis_and_duplicate_flags() {
    for args in [
        vec!["verify", "examples/features.neoil", "--gc-stats"],
        vec!["run", "examples/features.neoil", "--gc-stats", "--gc-stats"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Unexpected argument --gc-stats"));
    }
}
