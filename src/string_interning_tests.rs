//! Exploration only: no production pool, guest API or automatic literal interning.
use super::StringValue;
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Clone, PartialEq, Eq)]
struct Entry(StringValue);
impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}
impl Borrow<str> for Entry {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Rejected {
    Entries,
    Bytes,
    Allocation,
}

/// Strong retention within one explicitly owned pool. Quotas are logical payload
/// limits, not a measurement of HashSet, Arc or allocator overhead.
struct Pool {
    entries: HashSet<Entry>,
    bytes: usize,
    max_entries: usize,
    max_bytes: usize,
}
impl Pool {
    fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashSet::new(),
            bytes: 0,
            max_entries,
            max_bytes,
        }
    }
    fn intern(&mut self, value: StringValue) -> Result<StringValue, Rejected> {
        // Existing hits must remain available even when insertion quotas are full.
        if let Some(found) = self.entries.get(value.as_str()) {
            return Ok(found.0.clone());
        }
        if self.entries.len() >= self.max_entries {
            return Err(Rejected::Entries);
        }
        let bytes = self
            .bytes
            .checked_add(value.len())
            .filter(|b| *b <= self.max_bytes)
            .ok_or(Rejected::Bytes)?;
        self.entries
            .try_reserve(1)
            .map_err(|_| Rejected::Allocation)?;
        self.entries.insert(Entry(value.clone()));
        self.bytes = bytes;
        Ok(value)
    }
}

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
