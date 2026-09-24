//! Immutable UTF-8 payload shared by VM value copies. This is not guest identity.
use std::{fmt, ops::Deref, sync::Arc};

/// Host representation of intrinsic String text.
///
/// Cloning shares immutable bytes. Construction from an owned String adopts its
/// buffer; conversion back to an owned String copies only when another owner exists.
/// Text is a leaf outside the tracing heap's object count, as owned text was before.
#[derive(Clone, PartialEq, Eq)]
pub struct StringValue(Arc<String>);

impl StringValue {
    /// Borrow the immutable UTF-8 text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    /// Recover owned text, copying bytes only if other shared owners remain.
    pub fn into_owned(self) -> String {
        Arc::try_unwrap(self.0).unwrap_or_else(|shared| shared.as_ref().clone())
    }
}
impl From<String> for StringValue {
    fn from(value: String) -> Self {
        Self(Arc::new(value))
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
