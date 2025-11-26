use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub recursive: bool,
    pub dry_run: bool,
    pub replacement: char,
    pub targets: Vec<String>,
}

#[derive(Debug)]
pub enum CliError {
    Message(String),
    Help,
}

pub fn print_usage(mut w: impl Write) -> io::Result<()> {
    writeln!(w, "Usage: sanitize_filenames [options] [FILES...]")?;
    writeln!(w)?;
    writeln!(w, "Options:")?;
    writeln!(
        w,
        "  -r, --recursive        Recursively sanitize directories and their contents"
    )?;
    writeln!(
        w,
        "  -n, --dry-run          Show actions without renaming files"
    )?;
    writeln!(
        w,
        "  -c, --replacement CHAR Replacement character to use (default: _)"
    )?;
    writeln!(
        w,
        "  -h, --help             Show this help message and exit"
    )?;
    writeln!(w)?;
    writeln!(
        w,
        "Provide one or more files or directories to sanitize their names in-place."
    )?;
    writeln!(
        w,
        "Use '--' to stop option parsing when filenames begin with '-'."
    )?;
    writeln!(w)?;
    writeln!(w, "Examples:")?;
    writeln!(
        w,
        "  # sanitize a single file in the current directory"
    )?;
    writeln!(w, "  sanitize_filenames \"My File.txt\"")?;
    writeln!(w)?;
    writeln!(w, "  # preview changes without renaming")?;
    writeln!(w, "  sanitize_filenames --dry-run \"My File.txt\"")?;
    writeln!(w)?;
    writeln!(
        w,
        "  # sanitize recursively and use '-' as the separator"
    )?;
    writeln!(
        w,
        "  sanitize_filenames --recursive --replacement - ~/Downloads"
    )?;
    writeln!(w)?;
    writeln!(
        w,
        "  # sanitize a file whose name starts with a dash"
    )?;
    writeln!(w, "  sanitize_filenames -- --weird name.mp3")?;
    Ok(())
}

fn validate_replacement(s: &str) -> Result<char, String> {
    if s.is_empty() {
        return Err("Replacement character cannot be empty".to_string());
    }

    let mut chars = s.chars();
    let ch = chars
        .next()
        .ok_or_else(|| "Replacement character cannot be empty".to_string())?;
    if chars.next().is_some() {
        return Err("Replacement character must be a single character".to_string());
    }

    // Mirror the Ruby script: disallow the path separator.
    let illegal = ['/'];
    if illegal.contains(&ch) {
        return Err(format!("Replacement character '{}' is not allowed", ch));
    }

    Ok(ch)
}

pub fn parse_args(args: &[String]) -> Result<Config, CliError> {
    let mut recursive = false;
    let mut dry_run = false;
    let mut replacement = '_';
    let mut targets: Vec<String> = Vec::new();

    let mut i = 0;
    let mut end_of_opts = false;

    while i < args.len() {
        let arg = &args[i];

        if end_of_opts {
            if arg != "." && arg != ".." {
                targets.push(arg.clone());
            }
            i += 1;
            continue;
        }

        if arg == "--" {
            end_of_opts = true;
            i += 1;
            continue;
        }

        if !arg.starts_with('-') || arg == "-" {
            if arg != "." && arg != ".." {
                targets.push(arg.clone());
            }
            i += 1;
            continue;
        }

        match arg.as_str() {
            "-h" | "--help" => {
                return Err(CliError::Help);
            }
            "-r" | "--recursive" => {
                recursive = true;
                i += 1;
            }
            "-n" | "--dry-run" => {
                dry_run = true;
                i += 1;
            }
            "-c" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    CliError::Message("Option '-c' requires an argument".to_string())
                })?;
                replacement =
                    validate_replacement(value).map_err(CliError::Message)?;
                i += 2;
            }
            "--replacement" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    CliError::Message(
                        "Option '--replacement' requires an argument".to_string(),
                    )
                })?;
                replacement =
                    validate_replacement(value).map_err(CliError::Message)?;
                i += 2;
            }
            _ => {
                if let Some(rest) = arg.strip_prefix("-c") {
                    if rest.is_empty() {
                        return Err(CliError::Message(
                            "Option '-c' requires an argument".to_string(),
                        ));
                    }
                    replacement = validate_replacement(rest)
                        .map_err(CliError::Message)?;
                    i += 1;
                } else if let Some(rest) = arg.strip_prefix("--replacement=") {
                    if rest.is_empty() {
                        return Err(CliError::Message(
                            "Option '--replacement' requires an argument"
                                .to_string(),
                        ));
                    }
                    replacement = validate_replacement(rest)
                        .map_err(CliError::Message)?;
                    i += 1;
                } else {
                    return Err(CliError::Message(format!(
                        "Unknown option: {arg}"
                    )));
                }
            }
        }
    }

    Ok(Config {
        recursive,
        dry_run,
        replacement,
        targets,
    })
}

fn has_dot(name: &str) -> bool {
    name.split('.').count() > 1
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

fn is_directory(path_str: &str) -> bool {
    Path::new(path_str).is_dir()
}

fn has_extension(path_str: &str) -> bool {
    has_dot(path_str) && !is_directory(path_str) && !is_hidden(path_str)
}

fn extract_extension(path_str: &str) -> String {
    if has_extension(path_str) {
        match path_str.rsplit('.').next() {
            Some(ext) => ext.to_string(),
            None => String::new(),
        }
    } else {
        String::new()
    }
}

fn sanitize_component(name: &str, replacement: char, extension: &str) -> String {
    // First pass: replace characters similar to the Ruby gsub chain.
    let mut tmp = String::with_capacity(name.len());
    for ch in name.chars() {
        let mapped = match ch {
            '×' => 'x',
            c if c.is_whitespace()
                || matches!(
                    c,
                    '.' | ',' | '"' | ':' | '?' | '\'' | '#'
                        | ';' | '&' | '*' | '\\'
                ) =>
            {
                replacement
            }
            '(' | ')' | '[' | ']' => replacement,
            _ => ch,
        };
        tmp.push(mapped);
    }

    // Collapse multiple replacement characters into one.
    let mut collapsed = String::with_capacity(tmp.len());
    let mut prev_was_repl = false;
    for ch in tmp.chars() {
        if ch == replacement {
            if !prev_was_repl {
                collapsed.push(ch);
                prev_was_repl = true;
            }
        } else {
            collapsed.push(ch);
            prev_was_repl = false;
        }
    }

    // Remove trailing "<replacement><extension>" (without dot) if present.
    if !extension.is_empty() {
        let suffix = format!("{replacement}{extension}");
        if collapsed.ends_with(&suffix) {
            let new_len = collapsed.len().saturating_sub(suffix.len());
            collapsed.truncate(new_len);
        }
    }

    collapsed
}

pub fn sanitized_filename(input_file: &str, replacement: char) -> String {
    let extension = extract_extension(input_file);

    let path = Path::new(input_file);
    let fname_os: &OsStr = path.file_name().unwrap_or_else(|| OsStr::new(""));
    let fname = fname_os.to_string_lossy();

    let mut result = sanitize_component(&fname, replacement, &extension);

    // Reattach any parent directories, if present.
    if let Some(parent) = path.parent() {
        let parent_str = parent.to_string_lossy();
        if !parent_str.is_empty() && parent_str != "." {
            let mut buf = PathBuf::from(parent_str.as_ref());
            buf.push(&result);
            result = buf.to_string_lossy().to_string();
        }
    }

    let mut final_path = result;
    if !extension.is_empty() {
        if !final_path.is_empty() {
            final_path.push('.');
        }
        final_path.push_str(&extension);
    }

    final_path
}

pub fn rename_path(old: &Path, new: &Path, dry_run: bool) -> io::Result<PathBuf> {
    if old == new {
        println!(
            "Old name and new name are the same for '{}'.  Not changing",
            old.display()
        );
        return Ok(new.to_path_buf());
    } else if !old.exists() {
        println!(
            "Old file name '{}' does not exist.  Skipping",
            old.display()
        );
        return Ok(old.to_path_buf());
    } else if new.exists() && old != new {
        println!(
            "New file name '{}' already exists!  Skipping",
            new.display()
        );
        return Ok(old.to_path_buf());
    }

    let action = if dry_run { "Would change" } else { "Changing" };
    println!("{action} '{}' to '{}'", old.display(), new.display());

    if !dry_run {
        fs::rename(old, new)?;
    }

    Ok(new.to_path_buf())
}

pub fn sanitize_directory_tree(
    path: &Path,
    dry_run: bool,
    replacement: char,
) -> io::Result<PathBuf> {
    if !path.exists() {
        println!(
            "Old file name '{}' does not exist.  Skipping",
            path.display()
        );
        return Ok(path.to_path_buf());
    }

    let meta = fs::symlink_metadata(path)?;
    let file_type = meta.file_type();

    if !(file_type.is_dir() && !file_type.is_symlink()) {
        let new_name =
            sanitized_filename(&path.to_string_lossy(), replacement);
        let new_path = PathBuf::from(new_name);
        return rename_path(path, &new_path, dry_run);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child_path = entry.path();
        let child_meta = fs::symlink_metadata(&child_path)?;
        let child_type = child_meta.file_type();

        if child_type.is_dir() && !child_type.is_symlink() {
            sanitize_directory_tree(&child_path, dry_run, replacement)?;
        } else {
            let new_name =
                sanitized_filename(&child_path.to_string_lossy(), replacement);
            let new_path = PathBuf::from(new_name);
            rename_path(&child_path, &new_path, dry_run)?;
        }
    }

    let new_name = sanitized_filename(&path.to_string_lossy(), replacement);
    let new_path = PathBuf::from(new_name);
    rename_path(path, &new_path, dry_run)
}

pub fn run_from_env() -> i32 {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = match parse_args(&args) {
        Ok(cfg) => cfg,
        Err(CliError::Help) => {
            let _ = print_usage(io::stdout());
            return 0;
        }
        Err(CliError::Message(msg)) => {
            eprintln!("{msg}");
            let _ = print_usage(io::stderr());
            return 1;
        }
    };

    if config.targets.is_empty() {
        eprintln!("No files or directories specified");
        let _ = print_usage(io::stderr());
        return 1;
    }

    if let Err(e) = run(config) {
        eprintln!("Error: {e}");
        return 1;
    }

    0
}

pub fn run(config: Config) -> io::Result<()> {
    for target in &config.targets {
        let path = Path::new(target);
        if config.recursive {
            let _ =
                sanitize_directory_tree(path, config.dry_run, config.replacement)?;
        } else {
            let new_name =
                sanitized_filename(target, config.replacement);
            let new_path = PathBuf::from(new_name);
            let _ = rename_path(path, &new_path, config.dry_run)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let mut base = env::temp_dir();
        let unique = format!(
            "sanitize_filenames_test_{}_{}",
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

    #[test]
    fn sanitized_basic_cases() {
        assert_eq!(sanitized_filename("×", '_'), "x");
        assert_eq!(sanitized_filename("Hello", '_'), "Hello");
        assert_eq!(sanitized_filename("hello.wav", '_'), "hello.wav");
        assert_eq!(sanitized_filename("Hello World", '_'), "Hello_World");
        assert_eq!(sanitized_filename("Hello.World", '_'), "Hello.World");
        assert_eq!(sanitized_filename("hello world.wav", '_'), "hello_world.wav");
        assert_eq!(sanitized_filename("Hello.world.wav", '_'), "Hello_world.wav");
        assert_eq!(
            sanitized_filename("hello? + world.wav", '_'),
            "hello_+_world.wav"
        );
        assert_eq!(
            sanitized_filename("Bart_banner_14_5_×_2_5_in.png", '_'),
            "Bart_banner_14_5_x_2_5_in.png"
        );
        assert_eq!(
            sanitized_filename("hello? &&*()#@+ world.wav", '_'),
            "hello_@+_world.wav"
        );
        assert_eq!(
            sanitized_filename("August Gold Q&A Audio.m4a.wav", '_'),
            "August_Gold_Q_A_Audio_m4a.wav"
        );
        assert_eq!(
            sanitized_filename("nested/dir/file name.txt", '_'),
            "nested/dir/file_name.txt"
        );
        assert_eq!(
            sanitized_filename("/absolute/path/Hello World.txt", '_'),
            "/absolute/path/Hello_World.txt"
        );
        assert_eq!(
            sanitized_filename("relative/./path/Hello World.txt", '_'),
            "relative/./path/Hello_World.txt"
        );
    }

    #[test]
    fn custom_replacement() {
        assert_eq!(
            sanitized_filename("Hello World.txt", '-'),
            "Hello-World.txt"
        );
    }

    #[test]
    fn cli_replacement_option() {
        let args = vec!["--replacement".to_string(), "-".to_string()];
        let cfg = parse_args(&args).expect("parse_args failed");
        assert_eq!(cfg.replacement, '-');
        assert!(cfg.targets.is_empty());
    }

    #[test]
    fn invalid_replacement_rejected() {
        let err = validate_replacement("/").unwrap_err();
        assert!(
            err.contains("Replacement character '/' is not allowed"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn recursive_directory() {
        let tmp = temp_dir();
        let root = tmp.join("dir one");
        let sub = root.join("sub dir");
        fs::create_dir_all(&sub).unwrap();
        let file = sub.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let sanitized_root =
            sanitize_directory_tree(&root, false, '_').unwrap();

        let expected_root = tmp.join("dir_one");
        let expected_sub = expected_root.join("sub_dir");
        let expected_file = expected_sub.join("file_name.txt");

        assert_eq!(sanitized_root, expected_root);
        assert!(expected_sub.is_dir());
        assert!(expected_file.is_file());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn recursive_custom_replacement() {
        let tmp = temp_dir();
        let root = tmp.join("dir one");
        let sub = root.join("sub dir");
        fs::create_dir_all(&sub).unwrap();
        let file = sub.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let sanitized_root =
            sanitize_directory_tree(&root, false, '-').unwrap();

        let expected_root = tmp.join("dir-one");
        let expected_sub = expected_root.join("sub-dir");
        let expected_file = expected_sub.join("file-name.txt");

        assert_eq!(sanitized_root, expected_root);
        assert!(expected_sub.is_dir());
        assert!(expected_file.is_file());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn recursive_directory_with_nested_content() {
        let tmp = temp_dir();
        let root = tmp.join("Root Dir");
        let child_one = root.join("Child One");
        let child_two = root.join("Second & Child");
        let grand_one = child_one.join("Grand Child(1)");
        let grand_two = child_two.join("Grand (Final)");

        fs::create_dir_all(&grand_one).unwrap();
        fs::create_dir_all(&grand_two).unwrap();

        let files = [
            root.join("Root File?.txt"),
            child_one.join("Clip (A).mov"),
            child_one.join("Clip (B).mov"),
            grand_one.join("Take #1.wav"),
            child_two.join("Audio (Draft).wav"),
            grand_two.join("Mix #2?.wav"),
        ];
        for path in &files {
            fs::write(path, "test").unwrap();
        }

        let sanitized_root =
            sanitize_directory_tree(&root, false, '_').unwrap();

        let expected_root =
            PathBuf::from(sanitized_filename(root.to_str().unwrap(), '_'));
        let expected_child_one = PathBuf::from(sanitized_filename(
            expected_root.join("Child One").to_str().unwrap(),
            '_',
        ));
        let expected_child_two = PathBuf::from(sanitized_filename(
            expected_root.join("Second & Child").to_str().unwrap(),
            '_',
        ));
        let expected_grand_one = PathBuf::from(sanitized_filename(
            expected_child_one.join("Grand Child(1)").to_str().unwrap(),
            '_',
        ));
        let expected_grand_two = PathBuf::from(sanitized_filename(
            expected_child_two.join("Grand (Final)").to_str().unwrap(),
            '_',
        ));

        let expected_files = [
            PathBuf::from(sanitized_filename(
                expected_root.join("Root File?.txt").to_str().unwrap(),
                '_',
            )),
            PathBuf::from(sanitized_filename(
                expected_child_one.join("Clip (A).mov").to_str().unwrap(),
                '_',
            )),
            PathBuf::from(sanitized_filename(
                expected_child_one.join("Clip (B).mov").to_str().unwrap(),
                '_',
            )),
            PathBuf::from(sanitized_filename(
                expected_grand_one.join("Take #1.wav").to_str().unwrap(),
                '_',
            )),
            PathBuf::from(sanitized_filename(
                expected_child_two.join("Audio (Draft).wav").to_str().unwrap(),
                '_',
            )),
            PathBuf::from(sanitized_filename(
                expected_grand_two.join("Mix #2?.wav").to_str().unwrap(),
                '_',
            )),
        ];

        assert_eq!(sanitized_root, expected_root);
        for dir in &[
            &expected_child_one,
            &expected_child_two,
            &expected_grand_one,
            &expected_grand_two,
        ] {
            assert!(dir.is_dir(), "expected directory {:?} to exist", dir);
        }
        for file in &expected_files {
            assert!(file.is_file(), "expected file {:?} to exist", file);
        }
        for dir in &[&root, &child_one, &child_two, &grand_one, &grand_two] {
            assert!(
                !dir.exists(),
                "expected original path {:?} to be gone",
                dir
            );
        }

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn dry_run_does_not_rename() {
        let tmp = temp_dir();
        let file = tmp.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let desired =
            PathBuf::from(sanitized_filename(file.to_str().unwrap(), '_'));
        let result = rename_path(&file, &desired, true).unwrap();

        assert_eq!(result, desired);
        assert!(file.exists());
        assert!(!desired.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn recursive_dry_run_does_not_rename() {
        let tmp = temp_dir();
        let root = tmp.join("dir one");
        let sub = root.join("sub dir");
        fs::create_dir_all(&sub).unwrap();
        let file = sub.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let sanitized_root =
            sanitize_directory_tree(&root, true, '_').unwrap();
        let expected_root =
            PathBuf::from(sanitized_filename(root.to_str().unwrap(), '_'));

        assert_eq!(sanitized_root, expected_root);
        assert!(root.exists());
        assert!(!expected_root.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn option_terminator_handles_dash_prefixed_filename() {
        let tmp = temp_dir();
        let file = tmp.join("-file name.txt");
        fs::write(&file, "test").unwrap();

        let file_str = file.to_str().unwrap().to_string();
        let args = vec![
            "--dry-run".to_string(),
            "--".to_string(),
            file_str.clone(),
        ];
        let cfg = parse_args(&args).expect("parse_args failed");

        assert!(cfg.dry_run);
        assert_eq!(cfg.replacement, '_');
        assert_eq!(cfg.targets, vec![file_str.clone()]);

        let desired = PathBuf::from(sanitized_filename(
            &file_str,
            cfg.replacement,
        ));
        rename_path(Path::new(&file_str), &desired, cfg.dry_run).unwrap();

        assert!(Path::new(&file_str).exists());
        assert!(!desired.exists());

        fs::remove_dir_all(tmp).unwrap();
    }
}
