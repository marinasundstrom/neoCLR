//! Shared services whose ownership must survive individual Task.Run contexts.
//! File locking is allowed only outside managed graph access. Interning never
//! waits on host work and may run under graph access. No file operation takes
//! the graph gate, preventing a reverse lock order during blocking I/O.
use crate::{Limits, invocation_budget::Budget};
use std::sync::{Arc, Mutex};

pub(crate) struct Invocation {
    pub(crate) limits: Limits,
    pub(crate) budget: Arc<Budget>,
    pub(crate) files: Mutex<crate::file_streams::Files>,
    pub(crate) interned: Mutex<crate::string_interning::Pool>,
}
impl Invocation {
    pub(crate) fn new(limits: Limits) -> Arc<Self> {
        Arc::new(Self {
            limits,
            budget: Budget::new(limits),
            files: Mutex::new(Default::default()),
            interned: Mutex::new(crate::string_interning::Pool::new(
                limits.intern_entries,
                limits.intern_bytes,
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Value, file_streams::Operation};

    #[test]
    fn native_contexts_share_intern_identity_and_quota() {
        let invocation = Invocation::new(Limits {
            intern_entries: 1,
            intern_bytes: 4,
            ..Default::default()
        });
        let other = invocation.clone();
        let first = std::thread::spawn(move || {
            other
                .interned
                .lock()
                .unwrap()
                .intern("same".into())
                .unwrap()
        })
        .join()
        .unwrap();
        let mut pool = invocation.interned.lock().unwrap();
        let second = pool.intern("same".into()).unwrap();
        assert!(first.same_owner(&second));
        assert_eq!(
            pool.intern("next".into()),
            Err(crate::string_interning::Rejected::Entries)
        );
    }

    #[test]
    fn native_contexts_share_file_position_and_close_state() {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "neoclr-shared-file-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&path, b"ab").unwrap();
        let invocation = Invocation::new(Limits::default());
        let Value::Erased(handle) = invocation
            .files
            .lock()
            .unwrap()
            .invoke(
                Operation::OpenRead,
                &[Value::String(path.to_str().unwrap().into())],
                &invocation.limits,
            )
            .unwrap()
        else {
            panic!()
        };
        let other = invocation.clone();
        let passed = *handle.clone();
        let read = std::thread::spawn(move || {
            other
                .files
                .lock()
                .unwrap()
                .invoke(Operation::Read, &[passed, Value::Int32(1)], &other.limits)
                .unwrap()
        })
        .join()
        .unwrap();
        assert_eq!(
            read,
            Value::Erased(Box::new(Value::Array {
                element: crate::metadata::Type::Byte,
                elements: vec![Value::Byte(b'a')]
            }))
        );
        let mut files = invocation.files.lock().unwrap();
        assert_eq!(
            files
                .invoke(Operation::Position, &[*handle.clone()], &invocation.limits)
                .unwrap(),
            Value::Erased(Box::new(Value::Int64(1)))
        );
        files
            .invoke(Operation::Close, &[*handle.clone()], &invocation.limits)
            .unwrap();
        drop(files);
        let other = invocation.clone();
        let result = std::thread::spawn(move || {
            other
                .files
                .lock()
                .unwrap()
                .invoke(Operation::Read, &[*handle, Value::Int32(1)], &other.limits)
                .unwrap()
        })
        .join()
        .unwrap();
        assert_eq!(result, Value::Erased(Box::new(Value::Byte(6))));
        std::fs::remove_file(path).unwrap();
    }
}
