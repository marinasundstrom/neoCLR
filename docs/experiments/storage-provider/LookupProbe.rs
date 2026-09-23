//! Host-side evidence for the next Storage API, not a public neoCLR implementation.
use std::{
    fs,
    io::Read,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "neoclr-lookup-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn lookup_distinguishes_missing_file_and_directory_without_opening_content() {
    let fixture = Fixture::new();
    let path = fixture.0.join("item");
    assert_eq!(
        fs::metadata(&path).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    fs::write(&path, b"data").unwrap();
    assert!(fs::metadata(&path).unwrap().is_file());
    assert!(fs::metadata(&fixture.0).unwrap().is_dir());
}

#[test]
fn metadata_is_an_observation_not_a_live_item() {
    let fixture = Fixture::new();
    let path = fixture.0.join("item");
    fs::write(&path, b"first").unwrap();
    let observed = fs::metadata(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(observed.len(), 5);
    assert_eq!(
        fs::metadata(&path).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    assert_eq!(
        fs::File::open(&path).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
}

#[test]
fn a_path_descriptor_and_an_open_handle_have_different_identity() {
    let fixture = Fixture::new();
    let path = fixture.0.join("item");
    fs::write(&path, b"first").unwrap();
    let mut opened = fs::File::open(&path).unwrap();
    fs::rename(&path, fixture.0.join("moved")).unwrap();
    fs::write(&path, b"replacement").unwrap();
    let mut original = String::new();
    opened.read_to_string(&mut original).unwrap();
    assert_eq!(original, "first");
    assert_eq!(fs::read_to_string(&path).unwrap(), "replacement");
}

#[test]
fn equal_logical_paths_in_different_roots_do_not_identify_the_same_item() {
    let first = Fixture::new();
    let second = Fixture::new();
    fs::write(first.0.join("item"), b"one").unwrap();
    fs::write(second.0.join("item"), b"two").unwrap();
    assert_ne!(
        fs::read(first.0.join("item")).unwrap(),
        fs::read(second.0.join("item")).unwrap()
    );
}

#[cfg(unix)]
#[test]
fn following_links_is_distinct_from_inspecting_the_link() {
    let fixture = Fixture::new();
    let target = fixture.0.join("target");
    let link = fixture.0.join("link");
    fs::write(&target, b"data").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(fs::metadata(&link).unwrap().is_file());
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    fs::remove_file(&target).unwrap();
    assert_eq!(
        fs::metadata(&link).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
}
