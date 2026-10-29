//! Native watcher provider. OS threads retain only owned paths/bytes, never guest
//! references or callbacks. The caller drains a bounded queue on its own thread.
use notify::{
    Event, EventKind, Watcher,
    event::{ModifyKind, RenameMode},
};
use std::{
    collections::{HashMap, VecDeque},
    ffi::{CStr, c_char},
    path::Path,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicI64, Ordering},
    },
};

const CAPACITY: usize = 1024;
const MAX_PACKET: usize = 65536;
const INVALID: i32 = -1;
const MISSING: i32 = -2;
const DENIED: i32 = -3;
const IO: i32 = -4;
const CLOSED: i32 = -5;
const OVERFLOW: i32 = -6;
const ENCODING: i32 = -7;
#[derive(Default)]
struct Queue {
    packets: VecDeque<Vec<u8>>,
    error: i32,
}
impl Queue {
    fn fail(&mut self, error: i32) {
        if error == OVERFLOW {
            self.packets.clear();
        }
        self.error = error;
    }
    fn push(&mut self, packet: Vec<u8>) {
        if self.error == OVERFLOW {
            return;
        }
        if self.packets.len() == CAPACITY || packet.len() > MAX_PACKET {
            self.fail(OVERFLOW);
        } else {
            self.packets.push_back(packet);
        }
    }
}
struct Subscription {
    _watcher: notify::RecommendedWatcher,
    queue: Arc<Mutex<Queue>>,
}
static WATCHES: OnceLock<Mutex<HashMap<i64, Subscription>>> = OnceLock::new();
static NEXT: AtomicI64 = AtomicI64::new(1);
fn watches() -> &'static Mutex<HashMap<i64, Subscription>> {
    WATCHES.get_or_init(Default::default)
}
fn io_error(error: &std::io::Error) -> i32 {
    match error.kind() {
        std::io::ErrorKind::NotFound => MISSING,
        std::io::ErrorKind::PermissionDenied => DENIED,
        _ => IO,
    }
}
fn packet(kind: u8, root: &Path, path: &Path, old: Option<&Path>) -> Result<Vec<u8>, i32> {
    let relative = path.strip_prefix(root).map_err(|_| IO)?;
    let old_relative = match old {
        Some(p) => p.strip_prefix(root).map_err(|_| IO)?,
        None => Path::new(""),
    };
    let mut bytes = vec![kind];
    for part in [path, relative, old.unwrap_or(Path::new("")), old_relative] {
        bytes.extend_from_slice(part.to_str().ok_or(ENCODING)?.as_bytes());
        bytes.push(0);
    }
    Ok(bytes)
}
fn matches_filter(pattern: &str, name: &str) -> bool {
    if pattern.is_empty() || pattern == "*" || pattern == "*.*" {
        return true;
    }
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    let (mut p, mut n, mut star, mut retry) = (0, 0, None, 0);
    while n < name.len() {
        let same = p < pattern.len()
            && (pattern[p] == '?'
                || if cfg!(windows) {
                    pattern[p].to_lowercase().eq(name[n].to_lowercase())
                } else {
                    pattern[p] == name[n]
                });
        if same {
            p += 1;
            n += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some(p);
            p += 1;
            retry = n;
        } else if let Some(s) = star {
            retry += 1;
            n = retry;
            p = s + 1;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == '*' {
        p += 1;
    }
    p == pattern.len()
}
fn deliver(queue: &Mutex<Queue>, root: &Path, filter: &str, result: notify::Result<Event>) {
    let Ok(mut queue) = queue.lock() else {
        return;
    };
    let event = match result {
        Ok(e) => e,
        Err(_) => {
            queue.fail(IO);
            return;
        }
    };
    if event.need_rescan() {
        queue.fail(OVERFLOW);
        return;
    }
    let kind = match event.kind {
        EventKind::Create(_) => 1,
        EventKind::Remove(_) => 2,
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) if event.paths.len() == 2 => 8,
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => 2,
        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => 1,
        EventKind::Modify(ModifyKind::Name(_)) => {
            queue.fail(IO);
            return;
        }
        EventKind::Modify(_) => 4,
        EventKind::Access(_) => return,
        _ => {
            queue.fail(IO);
            return;
        }
    };
    let paths: Vec<(&Path, Option<&Path>)> = if kind == 8 {
        vec![(event.paths[1].as_path(), Some(event.paths[0].as_path()))]
    } else {
        event.paths.iter().map(|p| (p.as_path(), None)).collect()
    };
    for (path, old) in paths {
        if path == root {
            if kind != 4 {
                queue.fail(IO);
            }
            continue;
        }
        let selected = |p: &Path| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| matches_filter(filter, n))
        };
        let new_match = selected(path);
        let old_match = old.and_then(selected);
        if new_match.is_none() || (old.is_some() && old_match.is_none()) {
            queue.fail(ENCODING);
            continue;
        }
        if new_match != Some(true) && old_match != Some(true) {
            continue;
        }
        match packet(kind, root, path, old) {
            Ok(p) => queue.push(p),
            Err(e) => queue.fail(e),
        }
    }
}
pub fn open(path: &str, filter: &str, recursive: bool) -> Result<i64, i32> {
    if !cfg!(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux"
    )) {
        return Err(-8);
    }
    if path.is_empty() {
        return Err(INVALID);
    }
    let root = std::fs::canonicalize(path).map_err(|e| io_error(&e))?;
    if !root.is_dir() {
        return Err(INVALID);
    }
    let queue = Arc::new(Mutex::new(Queue::default()));
    let destination = queue.clone();
    let callback_root = root.clone();
    let filter = filter.to_owned();
    let mut watcher = notify::recommended_watcher(move |event| {
        deliver(&destination, &callback_root, &filter, event)
    })
    .map_err(|_| IO)?;
    watcher
        .watch(
            &root,
            if recursive {
                notify::RecursiveMode::Recursive
            } else {
                notify::RecursiveMode::NonRecursive
            },
        )
        .map_err(|_| IO)?;
    let id = NEXT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| IO)?;
    let mut table = watches().lock().map_err(|_| IO)?;
    if table.len() >= 64 {
        return Err(-9);
    }
    table.insert(
        id,
        Subscription {
            _watcher: watcher,
            queue,
        },
    );
    Ok(id)
}
pub fn close(handle: i64) {
    // Drop/join the OS watcher outside the table lock.
    let subscription = watches().lock().ok().and_then(|mut w| w.remove(&handle));
    drop(subscription);
}
pub fn read(handle: i64, output: &mut [u8]) -> i32 {
    let queue = match watches().lock() {
        Ok(w) => match w.get(&handle) {
            Some(w) => w.queue.clone(),
            None => return CLOSED,
        },
        Err(_) => return IO,
    };
    let Ok(mut queue) = queue.lock() else {
        return IO;
    };
    if queue.error != 0 {
        let error = queue.error;
        queue.error = 0;
        return error;
    }
    let Some(packet) = queue.packets.front() else {
        return 0;
    };
    if output.len() < packet.len() {
        return INVALID;
    }
    let count = packet.len();
    output[..count].copy_from_slice(packet);
    queue.packets.pop_front();
    count as i32
}
// C ABI calls never unwind. Handles are process-unique IDs, not pointers.
/// # Safety
/// `path` and `filter` must point to valid NUL-terminated UTF-8 strings for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_filewatch_open_v1(
    path: *const c_char,
    filter: *const c_char,
    recursive: i32,
) -> i64 {
    std::panic::catch_unwind(|| {
        if path.is_null() || filter.is_null() || !(0..=1).contains(&recursive) {
            return INVALID as i64;
        }
        let Ok(path) = (unsafe { CStr::from_ptr(path) }).to_str() else {
            return ENCODING as i64;
        };
        let Ok(filter) = (unsafe { CStr::from_ptr(filter) }).to_str() else {
            return ENCODING as i64;
        };
        if filter.contains(['/', '\\']) {
            return INVALID as i64;
        }
        if !cfg!(any(
            target_os = "macos",
            target_os = "windows",
            target_os = "linux"
        )) {
            return -8;
        }
        open(path, filter, recursive != 0).unwrap_or_else(i64::from)
    })
    .unwrap_or(IO as i64)
}
/// # Safety
/// `output` must be writable for `capacity` bytes; it is never retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_filewatch_read_v1(
    handle: i64,
    output: *mut u8,
    capacity: i32,
) -> i32 {
    std::panic::catch_unwind(|| {
        if output.is_null() || !(1..=MAX_PACKET as i32).contains(&capacity) {
            return INVALID;
        }
        read(handle, unsafe {
            std::slice::from_raw_parts_mut(output, capacity as usize)
        })
    })
    .unwrap_or(IO)
}
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_filewatch_close_v1(handle: i64) {
    let _ = std::panic::catch_unwind(|| close(handle));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        time::{Duration, Instant},
    };
    #[test]
    fn basename_filters_match_unicode_and_rename_either_side() {
        assert!(matches_filter("*.json", "café.json"));
        assert!(matches_filter("?.json", "é.json"));
        assert!(matches_filter("*.*", "README"));
        assert!(!matches_filter("*.json", "appsettings.txt"));
        let root = Path::new("/test");
        let queue = Mutex::new(Queue::default());
        deliver(
            &queue,
            root,
            "*.json",
            Ok(
                Event::new(EventKind::Create(notify::event::CreateKind::File))
                    .add_path(root.join("ignored.txt")),
            ),
        );
        assert!(queue.lock().unwrap().packets.is_empty());
        deliver(
            &queue,
            root,
            "*.json",
            Ok(
                Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
                    .add_path(root.join("settings.json"))
                    .add_path(root.join("settings.bak")),
            ),
        );
        assert_eq!(queue.lock().unwrap().packets.len(), 1);
    }
    #[test]
    fn rescan_and_unpaired_ambiguous_rename_report_loss() {
        let root = Path::new("/test");
        let queue = Mutex::new(Queue::default());
        deliver(
            &queue,
            root,
            "*",
            Ok(
                Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Any)))
                    .add_path(root.join("settings.json")),
            ),
        );
        assert_eq!(queue.lock().unwrap().error, IO);
        let mut event = Event::new(EventKind::Any);
        event.attrs.set_flag(notify::event::Flag::Rescan);
        deliver(&queue, root, "*", Ok(event));
        assert_eq!(queue.lock().unwrap().error, OVERFLOW);
    }
    #[test]
    fn ambiguous_rename_preserves_known_changes_and_root_metadata_is_ignored() {
        let root = Path::new("/test");
        let queue = Mutex::new(Queue::default());
        let changed =
            || Event::new(EventKind::Modify(ModifyKind::Any)).add_path(root.join("settings.json"));
        deliver(&queue, root, "*", Ok(changed()));
        deliver(
            &queue,
            root,
            "*",
            Ok(
                Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Any)))
                    .add_path(root.join("settings.json")),
            ),
        );
        deliver(&queue, root, "*", Ok(changed()));
        deliver(
            &queue,
            root,
            "*",
            Ok(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(root.to_path_buf())),
        );
        let queue = queue.lock().unwrap();
        assert_eq!(queue.error, IO);
        assert_eq!(queue.packets.len(), 2);
    }
    #[test]
    fn overflow_is_reported_and_queue_recovers() {
        let mut q = Queue::default();
        for _ in 0..=CAPACITY {
            q.push(vec![1]);
        }
        assert_eq!(q.error, OVERFLOW);
        assert!(q.packets.is_empty());
        q.error = 0;
        q.push(vec![2]);
        assert_eq!(q.packets.pop_front(), Some(vec![2]));
    }
    #[test]
    fn rename_preserves_both_names_and_unicode() {
        let root = Path::new("/test");
        let q = Mutex::new(Queue::default());
        deliver(
            &q,
            root,
            "*",
            Ok(
                Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
                    .add_path(root.join("old.json"))
                    .add_path(root.join("café.json")),
            ),
        );
        assert_eq!(
            q.lock().unwrap().packets.pop_front().unwrap(),
            packet(
                8,
                root,
                &root.join("café.json"),
                Some(&root.join("old.json"))
            )
            .unwrap()
        );
    }
    #[test]
    fn native_watch_reports_changes_and_releases_handles() {
        let root: PathBuf =
            std::env::temp_dir().join(format!("neoclr-watch-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let id = open(root.to_str().unwrap(), "appsettings.json", false).unwrap();
        std::fs::write(root.join("appsettings.json"), "{}").unwrap();
        let until = Instant::now() + Duration::from_secs(8);
        let mut buffer = vec![0; MAX_PACKET];
        let mut seen = false;
        while Instant::now() < until {
            let count = read(id, &mut buffer);
            if count > 0
                && buffer[..count as usize]
                    .windows(16)
                    .any(|s| s == b"appsettings.json")
            {
                seen = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        close(id);
        close(id);
        assert_eq!(read(id, &mut buffer), CLOSED);
        std::fs::remove_dir_all(root).unwrap();
        assert!(seen);
    }
}
