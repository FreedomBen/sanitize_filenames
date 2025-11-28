use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir() -> PathBuf {
    let mut base = env::temp_dir();
    let unique = format!(
        "sanitize_filenames_integration_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    base.push(unique);
    fs::create_dir_all(&base).unwrap();
    base
}

/// Build a nested directory tree with a wide variety of characters
/// and verify that recursive sanitization renames everything on disk
/// according to `sanitized_filename`.
#[test]
fn recursively_sanitizes_diverse_characters() {
    let tmp = temp_dir();

    let root = tmp.join("Root Dir (Audio) #1");
    let child_one = root.join("[Child Project] && Mixes?");
    let child_two = root.join("Second-Child (Drafts) #2");
    let grand_one = child_one.join("Grand ?Child* [v1]");
    let grand_two = child_two.join("Grand Child×Final (Take #1)");

    for dir in [&root, &child_one, &child_two, &grand_one, &grand_two] {
        fs::create_dir_all(dir).unwrap();
    }

    let files = [
        root.join(" Root File?.txt"),
        root.join("Another.File, With: Punctuation?.wav"),
        child_one.join("Clip (A) \"first\".wav"),
        child_one.join("Clip (B) #2?.wav"),
        grand_one.join("Take 1 × 2.m4a.wav"),
        grand_one.join("Interview Q&A (raw).mp3"),
        child_two.join("Audio (Draft); version 1?.wav"),
        child_two.join("Audio (Final)*mix&.wav"),
        grand_two.join("Mix #2 && final?.wav"),
        grand_two.join("Weird \\\\ slash.wav"),
    ];

    for path in &files {
        fs::write(path, "test").unwrap();
    }

    let sanitized_root =
        sanitize_filenames::sanitize_directory_tree(&root, false, '_')
            .unwrap();

    let expected_root = PathBuf::from(sanitize_filenames::sanitized_filename(
        root.to_str().unwrap(),
        '_',
    ));
    let expected_child_one = PathBuf::from(sanitize_filenames::sanitized_filename(
        expected_root.join("[Child Project] && Mixes?").to_str().unwrap(),
        '_',
    ));
    let expected_child_two = PathBuf::from(sanitize_filenames::sanitized_filename(
        expected_root.join("Second-Child (Drafts) #2").to_str().unwrap(),
        '_',
    ));
    let expected_grand_one =
        PathBuf::from(sanitize_filenames::sanitized_filename(
            expected_child_one.join("Grand ?Child* [v1]").to_str().unwrap(),
            '_',
        ));
    let expected_grand_two =
        PathBuf::from(sanitize_filenames::sanitized_filename(
            expected_child_two
                .join("Grand Child×Final (Take #1)")
                .to_str()
                .unwrap(),
            '_',
        ));

    assert_eq!(sanitized_root, expected_root);
    assert!(expected_root.is_dir());
    assert!(expected_child_one.is_dir());
    assert!(expected_child_two.is_dir());
    assert!(expected_grand_one.is_dir());
    assert!(expected_grand_two.is_dir());

    for original in [&root, &child_one, &child_two, &grand_one, &grand_two] {
        assert!(
            !original.exists(),
            "expected original path {:?} to be gone",
            original
        );
    }

    for original in &files {
        let rel = original.strip_prefix(&root).unwrap();
        let mut expected = expected_root.clone();
        for comp in rel.components() {
            let joined = expected.join(comp);
            expected = PathBuf::from(sanitize_filenames::sanitized_filename(
                joined.to_str().unwrap(),
                '_',
            ));
        }
        assert!(
            expected.is_file(),
            "expected sanitized file {:?} to exist",
            expected
        );
        assert!(
            !original.exists(),
            "expected original file {:?} to be gone",
            original
        );
    }

    fs::remove_dir_all(tmp).unwrap();
}

/// Ensure that every ASCII character that is valid in Linux filenames
/// (all bytes 1–127 except '/') is handled correctly when used inside
/// a filename, by creating one file per character and validating the
/// on-disk result matches `sanitized_filename`.
#[test]
fn recursively_sanitizes_all_ascii_filename_characters() {
    let tmp = temp_dir();
    let root = tmp.join("Ascii Chars Root");
    fs::create_dir_all(&root).unwrap();

    let mut originals = Vec::new();

    for byte in 1u8..=127 {
        let ch = byte as char;
        if ch == '/' {
            continue;
        }

        let filename = format!("id{byte:03}_{ch}.txt");
        let path = root.join(&filename);
        fs::write(&path, "test").unwrap();
        originals.push(path);
    }

    let sanitized_root =
        sanitize_filenames::sanitize_directory_tree(&root, false, '_')
            .unwrap();

    let expected_root = PathBuf::from(sanitize_filenames::sanitized_filename(
        root.to_str().unwrap(),
        '_',
    ));

    assert_eq!(sanitized_root, expected_root);
    assert!(expected_root.is_dir());
    assert!(
        !root.exists(),
        "expected original root directory {:?} to be gone",
        root
    );

    for original in &originals {
        let rel = original.strip_prefix(&root).unwrap();
        let mut expected = expected_root.clone();
        for comp in rel.components() {
            let joined = expected.join(comp);
            expected = PathBuf::from(sanitize_filenames::sanitized_filename(
                joined.to_str().unwrap(),
                '_',
            ));
        }
        assert!(
            expected.is_file(),
            "expected sanitized file {:?} to exist for original {:?}",
            expected,
            original
        );
        assert!(
            !original.exists(),
            "expected original file {:?} to be gone",
            original
        );
    }

    fs::remove_dir_all(tmp).unwrap();
}
