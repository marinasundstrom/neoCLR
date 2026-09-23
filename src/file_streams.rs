//! Invocation-owned, blocking regular-file resources for the file-stream experiment.
//! Integer handles are private runtime transport, not OS descriptors or capabilities.
use crate::{Fault, Limits, Value, metadata::Type};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
};

const MAX_OPEN_FILES: usize = 64;
const MAX_TRANSFER: usize = 64 * 1024;

#[derive(Clone, Copy)]
pub(crate) enum Operation {
    OpenRead,
    OpenWrite,
    CreateNew,
    Read,
    ReadInto,
    Write,
    Flush,
    Close,
    Kind,
    CreateDirectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum Error {
    InvalidPath = 1,
    NotFound,
    AccessDenied,
    WrongKind,
    AlreadyExists,
    Closed,
    InvalidRange,
    LimitExceeded,
    WrongAccess,
    Io,
}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match error.kind() {
            ErrorKind::NotFound => Self::NotFound,
            ErrorKind::PermissionDenied => Self::AccessDenied,
            ErrorKind::AlreadyExists => Self::AlreadyExists,
            ErrorKind::InvalidInput => Self::InvalidPath,
            ErrorKind::IsADirectory | ErrorKind::NotADirectory => Self::WrongKind,
            _ => Self::Io,
        }
    }
}
struct OpenFile {
    file: File,
    writable: bool,
}
#[derive(Default)]
pub(crate) struct Files {
    open: HashMap<i32, OpenFile>,
    next: i32,
}
fn path(path: &str) -> Result<&std::path::Path, Error> {
    if path.is_empty() || path.contains('\0') {
        Err(Error::InvalidPath)
    } else {
        Ok(std::path::Path::new(path))
    }
}
fn regular(path: &std::path::Path) -> Result<(), Error> {
    if path.metadata()?.is_file() {
        Ok(())
    } else {
        Err(Error::WrongKind)
    }
}
impl Files {
    fn open(&mut self, name: &str, mode: Operation) -> Result<i32, Error> {
        let path = path(name)?;
        if self.open.len() >= MAX_OPEN_FILES || self.next == i32::MAX {
            return Err(Error::LimitExceeded);
        }
        let writable = !matches!(mode, Operation::OpenRead);
        let file = if matches!(mode, Operation::CreateNew) {
            // Exclusive creation never replaces an existing file or follows an existing link.
            OpenOptions::new().write(true).create_new(true).open(path)?
        } else {
            // Reject known devices/directories before open, then validate the actual handle.
            // This is not a sandbox or a race-free defence against path replacement.
            regular(path)?;
            OpenOptions::new()
                .read(!writable)
                .write(writable)
                .open(path)?
        };
        if !file.metadata()?.is_file() {
            return Err(Error::WrongKind);
        }
        self.next += 1;
        self.open.insert(self.next, OpenFile { file, writable });
        Ok(self.next)
    }
    fn get(&mut self, id: i32, writable: bool) -> Result<&mut File, Error> {
        let entry = self.open.get_mut(&id).ok_or(Error::Closed)?;
        if entry.writable != writable {
            return Err(Error::WrongAccess);
        }
        Ok(&mut entry.file)
    }
    fn read(&mut self, id: i32, count: i32) -> Result<Vec<u8>, Error> {
        let count = usize::try_from(count).map_err(|_| Error::InvalidRange)?;
        if count > MAX_TRANSFER {
            return Err(Error::LimitExceeded);
        }
        let file = self.get(id, false)?;
        let mut bytes = vec![0; count];
        let read = loop {
            match file.read(&mut bytes) {
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                result => break result?,
            }
        };
        bytes.truncate(read);
        Ok(bytes)
    }
    fn write(&mut self, id: i32, bytes: &[u8]) -> Result<i32, Error> {
        if bytes.len() > MAX_TRANSFER {
            return Err(Error::LimitExceeded);
        }
        let file = self.get(id, true)?;
        loop {
            match file.write(bytes) {
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                result => return Ok(result? as i32),
            }
        }
    }
    fn close(&mut self, id: i32) -> Result<i32, Error> {
        if id <= 0 || id > self.next {
            return Err(Error::Closed);
        }
        // Idempotent release; IDs never recycle within an invocation.
        self.open.remove(&id);
        Ok(0)
    }
    pub(crate) fn invoke(
        &mut self,
        op: Operation,
        args: &[Value],
        limits: &Limits,
    ) -> Result<Value, Fault> {
        let result = match (op, args) {
            (
                mode @ (Operation::OpenRead | Operation::OpenWrite | Operation::CreateNew),
                [Value::String(name)],
            ) => self.open(name, mode).map(Value::Int32),
            (Operation::Read, [Value::Int32(id), Value::Int32(count)]) => {
                // Refuse a read before advancing the file if its output cannot fit the guest limit.
                if *count >= 0
                    && (*count as usize > limits.array_elements
                        || *count as usize > limits.array_bytes)
                {
                    Err(Error::LimitExceeded)
                } else {
                    match self.read(*id, *count) {
                        Ok(bytes) => Ok(crate::reflection::array(
                            "Byte",
                            bytes.into_iter().map(|b| Ok(Value::Byte(b))),
                            limits,
                        )?),
                        Err(error) => Err(error),
                    }
                }
            }
            (
                Operation::ReadInto,
                [
                    Value::Int32(id),
                    Value::ObjectReference(array),
                    Value::Int32(offset),
                    Value::Int32(count),
                ],
            ) => {
                if array.reference.target() != &Type::Array(Box::new(Type::Byte)) {
                    return Err(Fault::new("File read requires a byte array"));
                }
                array.reference.require_writable()?;
                let length = array.reference.array_length()?;
                match (usize::try_from(*offset), usize::try_from(*count)) {
                    (Ok(offset), Ok(count)) if offset <= length && count <= length - offset => {
                        match self.read(*id, count as i32) {
                            Ok(bytes) => {
                                // The blocking call cannot suspend or run guest code. Mutate only
                                // transferred elements, preserving aliases and the untouched tail.
                                for (index, byte) in bytes.iter().enumerate() {
                                    array
                                        .reference
                                        .element(offset + index, &Type::Byte)?
                                        .write(Value::Byte(*byte))?;
                                }
                                Ok(Value::Int32(bytes.len() as i32))
                            }
                            Err(error) => Err(error),
                        }
                    }
                    _ => Err(Error::InvalidRange),
                }
            }
            (
                Operation::Write,
                [
                    Value::Int32(id),
                    Value::ObjectReference(array),
                    Value::Int32(offset),
                    Value::Int32(count),
                ],
            ) => {
                let Value::Array {
                    element: Type::Byte,
                    elements,
                } = array.reference.read()?
                else {
                    return Err(Fault::new("File write requires a byte array"));
                };
                match (usize::try_from(*offset), usize::try_from(*count)) {
                    (Ok(offset), Ok(count))
                        if offset <= elements.len() && count <= elements.len() - offset =>
                    {
                        if count > MAX_TRANSFER {
                            Err(Error::LimitExceeded)
                        } else {
                            let bytes = elements[offset..offset + count]
                                .iter()
                                .map(|v| match v {
                                    Value::Byte(b) => Ok(*b),
                                    _ => Err(Fault::new("File write requires initialized bytes")),
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            self.write(*id, &bytes).map(Value::Int32)
                        }
                    }
                    _ => Err(Error::InvalidRange),
                }
            }
            (Operation::Flush, [Value::Int32(id)]) => self.get(*id, true).and_then(|file| {
                file.flush()?;
                Ok(Value::Int32(0))
            }),
            (Operation::Close, [Value::Int32(id)]) => self.close(*id).map(Value::Int32),
            (Operation::Kind, [Value::String(name)]) => path(name).and_then(|p| {
                let metadata = p.metadata()?;
                if metadata.is_file() {
                    Ok(Value::Int32(1))
                } else if metadata.is_dir() {
                    Ok(Value::Int32(2))
                } else {
                    Err(Error::WrongKind)
                }
            }),
            (Operation::CreateDirectory, [Value::String(name)]) => path(name).and_then(|p| {
                std::fs::create_dir(p)?;
                Ok(Value::Int32(0))
            }),
            _ => return Err(Fault::new("Invalid file resource service arguments")),
        };
        // Bootstrap-only tagged transport: Byte is an error; all successes have other types.
        Ok(Value::Erased(Box::new(
            result.unwrap_or_else(|e| Value::Byte(e as u8)),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "neoclr-stream-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn path(&self, name: &str) -> String {
            self.0.join(name).to_str().unwrap().into()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn round_trip_partial_reads_eof_and_close() {
        let fixture = Fixture::new();
        let path = fixture.path("data");
        let mut files = Files::default();
        let writer = files.open(&path, Operation::CreateNew).unwrap();
        assert_eq!(files.write(writer, b"hello"), Ok(5));
        assert_eq!(files.read(writer, 1), Err(Error::WrongAccess));
        files.get(writer, true).unwrap().flush().unwrap();
        assert_eq!(files.close(writer), Ok(0));
        let reader = files.open(&path, Operation::OpenRead).unwrap();
        assert_ne!(writer, reader);
        assert_eq!(files.read(reader, 2).unwrap(), b"he");
        assert_eq!(files.read(reader, 0).unwrap(), b"");
        assert_eq!(files.read(reader, 4).unwrap(), b"llo");
        assert_eq!(files.read(reader, 4).unwrap(), b"");
        assert_eq!(files.write(reader, b"x"), Err(Error::WrongAccess));
        assert_eq!(files.close(writer), Ok(0));
        assert_eq!(files.read(writer, 1), Err(Error::Closed));
        assert_eq!(files.read(reader, 1).unwrap(), b"");
        files.close(reader).unwrap();
    }
    #[test]
    fn creation_is_exclusive_and_write_open_does_not_truncate() {
        let fixture = Fixture::new();
        let path = fixture.path("data");
        std::fs::write(&path, b"original").unwrap();
        let mut files = Files::default();
        assert_eq!(
            files.open(&path, Operation::CreateNew),
            Err(Error::AlreadyExists)
        );
        let writer = files.open(&path, Operation::OpenWrite).unwrap();
        files.write(writer, b"NEW").unwrap();
        files.close(writer).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"NEWginal");
    }
    #[test]
    fn invalid_paths_ranges_and_limits_do_not_advance_file() {
        let fixture = Fixture::new();
        let path = fixture.path("data");
        std::fs::write(&path, b"abc").unwrap();
        let mut files = Files::default();
        assert_eq!(files.open("", Operation::OpenRead), Err(Error::InvalidPath));
        assert_eq!(
            files.open("x\0y", Operation::OpenRead),
            Err(Error::InvalidPath)
        );
        assert_eq!(
            files.open(&fixture.path("absent"), Operation::OpenRead),
            Err(Error::NotFound)
        );
        assert_eq!(
            files.open(fixture.0.to_str().unwrap(), Operation::OpenRead),
            Err(Error::WrongKind)
        );
        let id = files.open(&path, Operation::OpenRead).unwrap();
        assert_eq!(files.read(id, -1), Err(Error::InvalidRange));
        assert_eq!(
            files.read(id, MAX_TRANSFER as i32 + 1),
            Err(Error::LimitExceeded)
        );
        assert_eq!(files.read(id, 1).unwrap(), b"a");
        assert_eq!(files.close(id + 1), Err(Error::Closed));
    }
    #[test]
    fn live_handle_limit_recovers_without_reusing_ids() {
        let fixture = Fixture::new();
        let path = fixture.path("data");
        std::fs::write(&path, b"abc").unwrap();
        let mut files = Files::default();
        for _ in 0..MAX_OPEN_FILES {
            files.open(&path, Operation::OpenRead).unwrap();
        }
        assert_eq!(
            files.open(&path, Operation::OpenRead),
            Err(Error::LimitExceeded)
        );
        files.close(1).unwrap();
        assert_eq!(
            files.open(&path, Operation::OpenRead).unwrap(),
            MAX_OPEN_FILES as i32 + 1
        );
        assert_eq!(files.read(1, 1), Err(Error::Closed));
    }
}
