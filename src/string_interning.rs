//! Strong interning scoped to one interpreter execution, including its callbacks.
use crate::StringValue;
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Entry(StringValue);
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
pub(crate) enum Rejected {
    Entries,
    Bytes,
    Allocation,
}

/// Strong retention within one explicitly owned pool. Quotas are logical payload
/// limits, not a measurement of HashSet, Arc or allocator overhead.
pub(crate) struct Pool {
    pub(crate) entries: HashSet<Entry>,
    pub(crate) bytes: usize,
    max_entries: usize,
    max_bytes: usize,
}
impl Pool {
    pub(crate) fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashSet::new(),
            bytes: 0,
            max_entries,
            max_bytes,
        }
    }
    pub(crate) fn intern(&mut self, value: StringValue) -> Result<StringValue, Rejected> {
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

impl Rejected {
    pub(crate) fn fault(self) -> crate::Fault {
        match self {
            Self::Entries => crate::Fault::coded(
                crate::FaultCode::InternPoolLimitExceeded,
                "String intern entry limit exceeded",
            ),
            Self::Bytes => crate::Fault::coded(
                crate::FaultCode::InternPoolLimitExceeded,
                "String intern UTF-8 payload limit exceeded",
            ),
            Self::Allocation => crate::Fault::new("String intern pool allocation failed"),
        }
    }
}
