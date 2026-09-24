//! Canonical text, quota and lifetime checks for the execution-owned pool.
use super::StringValue;
use crate::string_interning::{Pool, Rejected};
use std::sync::Arc;

#[test]
fn repeated_log_field_names_share_canonical_owners() {
    // Parse field names into fresh strings from 1,000 structured log records.
    let names = ["timestamp", "level", "service", "message"];
    let mut pool = Pool::new(4, 28);
    let canonical: Vec<_> = names
        .iter()
        .map(|name| pool.intern((*name).into()).unwrap())
        .collect();
    let mut occurrences = 0;
    for _ in 0..1_000 {
        let record = "timestamp=2026-09-24T12:00:00Z level=info service=orders message=accepted";
        for (index, field) in record.split_whitespace().enumerate() {
            let (name, _value) = field.split_once('=').unwrap();
            let parsed = StringValue::from(name);
            let original = parsed.clone();
            assert!(!parsed.same_owner(&canonical[index]));
            let interned = pool.intern(parsed).unwrap();
            assert!(interned.same_owner(&canonical[index]));
            assert_eq!(interned.identity_hash(), canonical[index].identity_hash());
            assert_eq!(interned, original);
            // Interning does not rewrite references already held by the caller.
            assert!(!original.same_owner(&interned));
            occurrences += 1;
        }
    }
    assert_eq!(pool.entries.len(), 4);
    assert_eq!(pool.bytes, 28);
    println!(
        "Repeated log fields: {occurrences} occurrences -> 4 retained text owners, 28 UTF-8 payload bytes"
    );
    println!(
        "Input strings are still allocated before interning; this is retained-payload sharing, not an allocation benchmark."
    );
}

#[test]
fn quotas_bound_new_entries_without_rejecting_existing_hits() {
    let mut pool = Pool::new(2, 2);
    let empty = pool.intern("".into()).unwrap();
    let accented = pool.intern("é".into()).unwrap();
    assert_eq!(pool.intern("x".into()), Err(Rejected::Entries));
    assert!(pool.intern("".into()).unwrap().same_owner(&empty));
    assert!(pool.intern("é".into()).unwrap().same_owner(&accented));
    assert_eq!((pool.entries.len(), pool.bytes), (2, 2));
    let mut bytes = Pool::new(3, 2);
    assert_eq!(bytes.intern("👩‍💻".into()), Err(Rejected::Bytes));
    assert_eq!((bytes.entries.len(), bytes.bytes), (0, 0));
    assert!(bytes.intern("é".into()).is_ok());
    assert_eq!(bytes.intern("x".into()), Err(Rejected::Bytes));
    assert_eq!((bytes.entries.len(), bytes.bytes), (1, 2));
    assert_eq!(Pool::new(0, 0).intern("".into()), Err(Rejected::Entries));
}

#[test]
fn pools_are_independent_and_do_not_normalize_text() {
    let mut first = Pool::new(8, 128);
    let mut second = Pool::new(8, 128);
    for text in ["", "a\0b", "é", "e\u{301}", "👩‍💻", "Name", "name"] {
        let left = first.intern(text.into()).unwrap();
        let right = second.intern(text.into()).unwrap();
        assert_eq!(left, right);
        assert!(!left.same_owner(&right));
        assert!(first.intern(text.into()).unwrap().same_owner(&left));
    }
    assert_eq!(first.entries.len(), 7);
    // Pool separation does not clone an already shared input into a new identity.
    let shared: StringValue = "shared".into();
    assert!(
        first
            .intern(shared.clone())
            .unwrap()
            .same_owner(&second.intern(shared).unwrap())
    );
}

#[test]
fn pool_retention_and_gc_have_separate_lifetimes() {
    use crate::{CollectionReason, ManagedHeap, Value, metadata::Type};
    let mut pool = Pool::new(2, 64);
    let canonical = pool.intern("retained".into()).unwrap();
    let observer = Arc::downgrade(&canonical.0);
    let hash = canonical.identity_hash();
    let mut heap = ManagedHeap::default();
    heap.allocate(Value::Array {
        element: Type::String,
        elements: vec![Value::String(canonical)],
    })
    .unwrap();
    heap.collect(vec![], CollectionReason::ExecutionCompleted)
        .unwrap();
    assert!(heap.is_empty());
    assert!(
        observer.upgrade().is_some(),
        "pool must retain its canonical text"
    );
    let host = pool.intern("retained".into()).unwrap();
    let pool_only = pool.intern("pool only".into()).unwrap();
    let pool_observer = Arc::downgrade(&pool_only.0);
    drop(pool_only);
    drop(pool);
    assert!(
        pool_observer.upgrade().is_none(),
        "pool-only text must be released"
    );
    assert_eq!(host.as_str(), "retained");
    assert_eq!(host.identity_hash(), hash);
    drop(host);
    assert!(
        observer.upgrade().is_none(),
        "host release must release the last owner"
    );
}

fn execute(
    program: &crate::LoadedProgram,
    input: StringValue,
    options: impl Into<crate::ExecutionOptions>,
) -> Result<crate::Execution, crate::Fault> {
    program
        .resolve_function(&crate::assembler::parse_function_ref("Test(String)").unwrap())
        .unwrap()
        .invoke(vec![crate::Value::String(input)], options)
}

#[test]
fn execution_interning_is_fresh_per_host_invocation_and_retains_returned_owners() {
    use super::ownership_tests::program;
    use crate::{Limits, Value};
    let p = program(
        "ldarg input\ncall neoCLR.Runtime.StringIntern(String)\nret",
        "",
        "String",
    );
    let limits = Limits {
        intern_entries: 1,
        intern_bytes: 4,
        ..Limits::default()
    };
    let left = execute(&p, "same".into(), limits).unwrap();
    let right = execute(&p, "same".into(), limits).unwrap();
    let (Value::String(a), Value::String(b)) = (&left.value, &right.value) else {
        panic!("expected text")
    };
    assert_eq!(a, b);
    assert!(
        !a.same_owner(b),
        "independent invocations unexpectedly shared a pool"
    );
    let host = a.clone();
    let hash = host.identity_hash();
    drop(left);
    drop(right);
    let repeated = execute(&p, host.clone(), limits).unwrap();
    let Value::String(returned) = repeated.value else {
        panic!("expected text")
    };
    assert!(
        returned.same_owner(&host),
        "explicitly shared host input lost its identity"
    );
    assert_eq!(returned.identity_hash(), hash);
}

#[test]
fn execution_interning_enforces_limits_and_keeps_existing_hits_usable() {
    use super::ownership_tests::program;
    use crate::{FaultCode, Limits, Value};
    let p = program(
        "ldarg input\ncall neoCLR.Runtime.StringIntern(String)\npop\nldstr \"same\"\ncall neoCLR.Runtime.StringIntern(String)\nret",
        "",
        "String",
    );
    let limits = Limits {
        intern_entries: 1,
        intern_bytes: 4,
        ..Limits::default()
    };
    let original: StringValue = "same".into();
    let result = execute(&p, original.clone(), limits).unwrap();
    let Value::String(returned) = result.value else {
        panic!("expected text")
    };
    assert!(original.same_owner(&returned));
    for (input, limits) in [
        ("else", limits),
        (
            "same",
            Limits {
                intern_entries: 0,
                ..limits
            },
        ),
        (
            "é",
            Limits {
                intern_bytes: 1,
                ..limits
            },
        ),
    ] {
        assert_eq!(
            execute(&p, input.into(), limits).unwrap_err().code,
            FaultCode::InternPoolLimitExceeded
        );
    }
    assert_eq!(
        FaultCode::InternPoolLimitExceeded.as_str(),
        "InternPoolLimitExceeded"
    );
}

#[test]
fn execution_pool_releases_unreturned_text_on_completion_fault_and_cancellation() {
    use super::ownership_tests::program;
    use crate::{CancellationToken, ExecutionOptions, FaultCode, Limits};
    #[derive(Debug)]
    struct CancelOnWrite(CancellationToken);
    impl crate::Console for CancelOnWrite {
        fn read_byte(&self) -> std::io::Result<Option<u8>> {
            Ok(None)
        }
        fn write_line(&self, _: &str) -> std::io::Result<()> {
            self.0.cancel();
            Ok(())
        }
    }
    for (tail, expected) in [
        ("ldstr \"done\"\nret", None),
        ("fault \"intentional\"", Some(FaultCode::UserFault)),
        (
            "ldstr \"cancel\"\ncall neoCLR.Runtime.WriteLine(String)\npop\nldstr \"done\"\nret",
            Some(FaultCode::ExecutionCancelled),
        ),
    ] {
        let p = program(
            &format!("ldarg input\ncall neoCLR.Runtime.StringIntern(String)\npop\n{tail}"),
            "",
            "String",
        );
        let token = CancellationToken::new();
        let options = ExecutionOptions {
            limits: Limits::default(),
            cancellation: Some(token.clone()),
            console: Some(Arc::new(CancelOnWrite(token))),
            ..Default::default()
        };
        let input: StringValue = "pool-owned".into();
        let observer = Arc::downgrade(&input.0);
        let result = execute(&p, input, options);
        if let Some(code) = expected {
            assert_eq!(result.unwrap_err().code, code);
        } else {
            assert!(result.is_ok());
        }
        assert!(
            observer.upgrade().is_none(),
            "pool retained input after {tail}"
        );
    }
}

#[test]
fn isolated_workers_receive_independent_intern_quotas() {
    use super::ownership_tests::program;
    use crate::{Limits, Value};
    let helper = ".function Canon(String input) -> String\nldarg input\ncall neoCLR.Runtime.StringIntern(String)\nret\n.end";
    for operation in ["StartWorker", "QueueWorker"] {
        let body = format!(
            "ldarg input\ncall neoCLR.Runtime.StringIntern(String)\npop\ndelegate.bind System.Func<String,String> = Canon(String)\nldstr \"child\"\ncall neoCLR.Runtime.{operation}(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)\nret"
        );
        let p = program(&body, helper, "String");
        let result = execute(
            &p,
            "parent".into(),
            Limits {
                intern_entries: 1,
                intern_bytes: 6,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(result.value, Value::String("child".into()));
    }
}

#[test]
fn interning_null_default_string_has_a_null_reference_fault() {
    let p = super::ownership_tests::program(
        ".local String missing\nldloca missing\ninitobj String\nldloc missing\ncall neoCLR.Runtime.StringIntern(String)\nret",
        "",
        "String",
    );
    let fault = execute(&p, "unused".into(), crate::Limits::default()).unwrap_err();
    assert_eq!(fault.code, crate::FaultCode::NullReference);
}
