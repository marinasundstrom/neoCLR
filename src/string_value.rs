//! Immutable UTF-8 payload with owner identity shared by VM value copies.
use std::{
    fmt,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

/// Host representation of intrinsic String text.
///
/// Cloning shares immutable bytes. Construction from an owned String adopts its
/// buffer; conversion back to an owned String copies only when another owner exists.
/// Text is a leaf outside the tracing heap's object count, as owned text was before.
/// Aliases share guest reference identity; independently constructed equal text does
/// not. Rust equality remains content-based. Extracting and reconstructing owned
/// text creates a new identity; hashes are process-local and may collide.
#[derive(Clone)]
pub struct StringValue(Arc<StringData>);

struct StringData {
    text: String,
    // Hash seed only: wrapping is harmless because hashes may collide.
    identity_hash: u32,
}
static NEXT_HASH: AtomicU32 = AtomicU32::new(1);
impl PartialEq for StringValue {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Eq for StringValue {}

impl StringValue {
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub(crate) fn identity_hash(&self) -> i32 {
        crate::object_identity::mix_hash(self.0.identity_hash as u64)
    }
    /// Borrow the immutable UTF-8 text.
    pub fn as_str(&self) -> &str {
        self.0.text.as_str()
    }
    /// Recover owned text, copying bytes only if other shared owners remain.
    pub fn into_owned(self) -> String {
        Arc::try_unwrap(self.0)
            .map(|data| data.text)
            .unwrap_or_else(|shared| shared.text.clone())
    }
}
impl From<String> for StringValue {
    fn from(value: String) -> Self {
        Self(Arc::new(StringData {
            text: value,
            identity_hash: NEXT_HASH.fetch_add(1, Ordering::Relaxed),
        }))
    }
}
impl From<&str> for StringValue {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}
impl Deref for StringValue {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<str> for StringValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl fmt::Debug for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
impl fmt::Display for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollectionReason, ManagedHeap, Value, metadata::Type};

    #[test]
    fn adopts_owned_buffer_and_copies_only_for_shared_extraction() {
        let owned = "hello 👩‍💻".to_owned();
        let pointer = owned.as_ptr();
        let text = StringValue::from(owned);
        assert_eq!(pointer, text.as_ptr());
        let peer = text.clone();
        assert!(Arc::ptr_eq(&text.0, &peer.0));
        let extracted = text.into_owned();
        assert_ne!(extracted.as_ptr(), pointer);
        assert_eq!(extracted, peer.as_str());
        let recovered = peer.into_owned();
        assert_eq!(recovered.as_ptr(), pointer);
    }

    #[test]
    fn equal_contents_do_not_require_a_shared_owner() {
        let left = StringValue::from("same");
        let right = StringValue::from("same");
        assert_eq!(left, right);
        assert!(!Arc::ptr_eq(&left.0, &right.0));
    }

    #[test]
    fn identity_hash_collisions_do_not_merge_distinct_owners() {
        let make = || {
            StringValue(Arc::new(StringData {
                text: "same".into(),
                identity_hash: 7,
            }))
        };
        let left = make();
        let right = make();
        assert_eq!(left, right);
        assert_eq!(left.identity_hash(), right.identity_hash());
        assert!(
            !crate::object_identity::reference_equals(&Value::String(left), &Value::String(right))
                .unwrap()
        );
    }

    #[test]
    fn tracing_collection_releases_array_text_but_retains_host_copy() {
        let text = StringValue::from("retained 👩‍💻");
        let weak = Arc::downgrade(&text.0);
        let mut heap = ManagedHeap::default();
        let id = heap
            .allocate(Value::Array {
                element: Type::String,
                elements: vec![Value::String(text.clone()), Value::String(text)],
            })
            .unwrap();
        heap.collect(vec![id], CollectionReason::AllocationPressure)
            .unwrap();
        let host_copy = heap.get(id).unwrap();
        heap.collect(vec![], CollectionReason::ExecutionCompleted)
            .unwrap();
        assert!(heap.is_empty());
        assert!(weak.upgrade().is_some());
        drop(heap);
        assert!(weak.upgrade().is_some());
        drop(host_copy);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn tracing_collection_releases_unretained_text() {
        let text = StringValue::from("temporary");
        let weak = Arc::downgrade(&text.0);
        let mut heap = ManagedHeap::default();
        heap.allocate(Value::String(text)).unwrap();
        heap.collect(vec![], CollectionReason::ExecutionCompleted)
            .unwrap();
        assert!(weak.upgrade().is_none());
    }
}

#[cfg(test)]
#[path = "string_ownership_tests.rs"]
mod ownership_tests;
