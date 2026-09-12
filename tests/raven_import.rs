use neoclr::{Limits, assemble, run, verify};

#[test]
fn raven_emitted_static_corpus_runs_against_real_system_library() {
    for (source, expected) in [
        (
            include_str!("../docs/experiments/raven-target/imported/CoreOnly.neoil"),
            vec!["Hello from Raven on neoCLR"],
        ),
        (
            include_str!("../docs/experiments/raven-target/imported/CoreEmpty.neoil"),
            vec![],
        ),
        (
            include_str!("../docs/experiments/raven-target/imported/CoreNested.neoil"),
            vec![],
        ),
        (
            include_str!("../docs/experiments/raven-target/imported/CoreInt32.neoil"),
            vec![],
        ),
    ] {
        let module = assemble(source).unwrap();
        verify(&module).unwrap();
        assert_eq!(run(&module, Limits::default()).unwrap().output, expected);
    }
}
