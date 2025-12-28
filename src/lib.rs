use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum SanitizeMode {
    Legacy,
    Full,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub recursive: bool,
    pub dry_run: bool,
    pub replacement: char,
    pub targets: Vec<String>,
    pub full_sanitize: bool,
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
        "  -F, --full-sanitize    Replace all non-alphanumeric characters (except '_' and '-')"
    )?;
    writeln!(
        w,
        "                          with the replacement character"
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
    let mut full_sanitize = false;
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
            "-F" | "--full-sanitize" => {
                full_sanitize = true;
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
        full_sanitize,
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

fn sanitize_component(
    name: &str,
    replacement: char,
    extension: &str,
    mode: SanitizeMode,
) -> String {
    // First pass: map characters according to the selected mode.
    let mut tmp = String::with_capacity(name.len());
    for ch in name.chars() {
        let mapped = match mode {
            SanitizeMode::Legacy => match ch {
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
            },
            SanitizeMode::Full => {
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                    ch
                } else {
                    replacement
                }
            }
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

    // Trim any leading or trailing replacement characters to avoid
    // introducing sanitized names that start or end with them.
    let trimmed = collapsed.trim_matches(replacement).to_string();
    if trimmed.is_empty() && !collapsed.is_empty() {
        // Preserve a single replacement character for inputs that were
        // entirely replaced so the filename does not become empty.
        collapsed.chars().next().into_iter().collect()
    } else {
        trimmed
    }
}

pub fn sanitized_filename(
    input_file: &str,
    replacement: char,
    mode: SanitizeMode,
) -> String {
    let extension = extract_extension(input_file);

    let path = Path::new(input_file);
    let fname_os: &OsStr = path.file_name().unwrap_or_else(|| OsStr::new(""));
    let fname = fname_os.to_string_lossy();

    let mut result =
        sanitize_component(&fname, replacement, &extension, mode);

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
    mode: SanitizeMode,
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
        let new_name = sanitized_filename(
            &path.to_string_lossy(),
            replacement,
            mode,
        );
        let new_path = PathBuf::from(new_name);
        return rename_path(path, &new_path, dry_run);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child_path = entry.path();
        let child_meta = fs::symlink_metadata(&child_path)?;
        let child_type = child_meta.file_type();

        if child_type.is_dir() && !child_type.is_symlink() {
            sanitize_directory_tree(&child_path, dry_run, replacement, mode)?;
        } else {
            let new_name = sanitized_filename(
                &child_path.to_string_lossy(),
                replacement,
                mode,
            );
            let new_path = PathBuf::from(new_name);
            rename_path(&child_path, &new_path, dry_run)?;
        }
    }

    let new_name =
        sanitized_filename(&path.to_string_lossy(), replacement, mode);
    let new_path = PathBuf::from(new_name);
    rename_path(path, &new_path, dry_run)
}

fn run_with_args(args: &[String]) -> i32 {
    let config = match parse_args(args) {
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

pub fn run_from_env() -> i32 {
    let args: Vec<String> = env::args().skip(1).collect();
    run_with_args(&args)
}

pub fn run(config: Config) -> io::Result<()> {
    let mode = if config.full_sanitize {
        SanitizeMode::Full
    } else {
        SanitizeMode::Legacy
    };

    for target in &config.targets {
        let path = Path::new(target);
        if config.recursive {
            let _ = sanitize_directory_tree(
                path,
                config.dry_run,
                config.replacement,
                mode,
            )?;
        } else {
            let new_name =
                sanitized_filename(target, config.replacement, mode);
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
    fn print_usage_includes_sections() {
        let mut buf: Vec<u8> = Vec::new();
        print_usage(&mut buf).expect("print_usage failed");
        let output = String::from_utf8(buf).expect("usage is valid UTF-8");
        assert!(output.contains("Usage: sanitize_filenames [options] [FILES...]"));
        assert!(output.contains("Options:"));
        assert!(output.contains("Examples:"));
    }

    #[test]
    fn has_dot_recognizes_periods() {
        assert!(!has_dot("file"));
        assert!(has_dot("file.txt"));
        assert!(has_dot(".hidden"));
        assert!(has_dot("a.b.c"));
    }

    #[test]
    fn is_hidden_detects_leading_dot_only() {
        assert!(is_hidden(".gitignore"));
        assert!(!is_hidden("file"));
        assert!(!is_hidden("dir/.git"));
    }

    #[test]
    fn is_directory_matches_filesystem() {
        let base = temp_dir();
        let dir = base.join("dir");
        let file = base.join("file.txt");

        fs::create_dir_all(&dir).unwrap();
        fs::write(&file, "test").unwrap();

        assert!(is_directory(dir.to_str().unwrap()));
        assert!(!is_directory(file.to_str().unwrap()));

        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn has_extension_ignores_dirs_and_hidden_files() {
        let base = temp_dir();
        let dir_with_dot = base.join("dir.with.dot");
        fs::create_dir_all(&dir_with_dot).unwrap();

        assert!(!has_extension(dir_with_dot.to_str().unwrap()));
        assert!(has_extension("file.txt"));
        assert!(has_extension("archive.tar.gz"));
        assert!(!has_extension(".gitignore"));

        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn extract_extension_handles_files_and_dirs() {
        let base = temp_dir();
        let dir_with_dot = base.join("dir.with.dot");
        fs::create_dir_all(&dir_with_dot).unwrap();

        assert_eq!(extract_extension("file.txt"), "txt");
        assert_eq!(extract_extension("archive.tar.gz"), "gz");
        assert_eq!(
            extract_extension(
                &format!("{}/{}", dir_with_dot.to_string_lossy(), "file.dat")
            ),
            "dat"
        );
        assert_eq!(extract_extension(dir_with_dot.to_str().unwrap()), "");
        assert_eq!(extract_extension(".gitignore"), "");

        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn sanitize_component_collapses_repeated_replacements() {
        let result = sanitize_component(
            "Hello   World",
            '_',
            "",
            SanitizeMode::Legacy,
        );
        assert_eq!(result, "Hello_World");
    }

    #[test]
    fn sanitize_component_maps_special_characters_and_trailing_extension() {
        let result = sanitize_component(
            "August Gold Q&A Audio.m4a.wav",
            '_',
            "wav",
            SanitizeMode::Legacy,
        );
        assert_eq!(result, "August_Gold_Q_A_Audio_m4a");
    }

    #[test]
    fn sanitize_component_maps_multiplication_sign() {
        let result =
            sanitize_component("size 4×4", '_', "", SanitizeMode::Legacy);
        assert_eq!(result, "size_4x4");
    }

    #[test]
    fn parse_args_sets_flags_and_targets() {
        let args = vec![
            "-r".to_string(),
            "--dry-run".to_string(),
            "file1".to_string(),
            ".".to_string(),
            "..".to_string(),
            "dir2".to_string(),
        ];
        let cfg = parse_args(&args).expect("parse_args failed");
        assert!(cfg.recursive);
        assert!(cfg.dry_run);
        assert_eq!(cfg.replacement, '_');
        assert!(!cfg.full_sanitize);
        assert_eq!(cfg.targets, vec!["file1".to_string(), "dir2".to_string()]);
    }

    #[test]
    fn parse_args_replacement_forms() {
        let args_short = vec!["-c".to_string(), "+".to_string(), "file".to_string()];
        let cfg_short = parse_args(&args_short).expect("parse_args failed");
        assert_eq!(cfg_short.replacement, '+');
        assert_eq!(cfg_short.targets, vec!["file".to_string()]);
        assert!(!cfg_short.full_sanitize);

        let args_short_inline = vec!["-c+".to_string(), "file".to_string()];
        let cfg_short_inline =
            parse_args(&args_short_inline).expect("parse_args failed");
        assert_eq!(cfg_short_inline.replacement, '+');
        assert_eq!(cfg_short_inline.targets, vec!["file".to_string()]);
        assert!(!cfg_short_inline.full_sanitize);

        let args_long_inline =
            vec!["--replacement=+".to_string(), "file".to_string()];
        let cfg_long_inline =
            parse_args(&args_long_inline).expect("parse_args failed");
        assert_eq!(cfg_long_inline.replacement, '+');
        assert_eq!(cfg_long_inline.targets, vec!["file".to_string()]);
        assert!(!cfg_long_inline.full_sanitize);
    }

    #[test]
    fn parse_args_missing_replacement_argument() {
        let args_short = vec!["-c".to_string()];
        match parse_args(&args_short) {
            Err(CliError::Message(msg)) => {
                assert!(msg.contains("Option '-c' requires an argument"))
            }
            _ => panic!("expected error for missing -c argument"),
        }

        let args_long = vec!["--replacement".to_string()];
        match parse_args(&args_long) {
            Err(CliError::Message(msg)) => {
                assert!(msg.contains("Option '--replacement' requires an argument"))
            }
            _ => panic!("expected error for missing --replacement argument"),
        }
    }

    #[test]
    fn parse_args_unknown_option_yields_error() {
        let args = vec!["--unknown".to_string()];
        match parse_args(&args) {
            Err(CliError::Message(msg)) => {
                assert!(msg.contains("Unknown option: --unknown"))
            }
            _ => panic!("expected error for unknown option"),
        }
    }

    #[test]
    fn parse_args_allows_single_dash_target() {
        let args = vec!["-".to_string()];
        let cfg = parse_args(&args).expect("parse_args failed");
        assert!(!cfg.full_sanitize);
        assert_eq!(cfg.targets, vec!["-".to_string()]);
    }

    #[test]
    fn parse_args_full_sanitize_flags() {
        let args = vec!["--full-sanitize".to_string(), "file".to_string()];
        let cfg = parse_args(&args).expect("parse_args failed");
        assert!(cfg.full_sanitize);
        assert_eq!(cfg.targets, vec!["file".to_string()]);

        let args_short = vec!["-F".to_string(), "other".to_string()];
        let cfg_short = parse_args(&args_short).expect("parse_args failed");
        assert!(cfg_short.full_sanitize);
        assert_eq!(cfg_short.targets, vec!["other".to_string()]);
    }

    #[test]
    fn sanitized_basic_cases() {
        assert_eq!(
            sanitized_filename("×", '_', SanitizeMode::Legacy),
            "x"
        );
        assert_eq!(
            sanitized_filename("Hello", '_', SanitizeMode::Legacy),
            "Hello"
        );
        assert_eq!(
            sanitized_filename("hello.wav", '_', SanitizeMode::Legacy),
            "hello.wav"
        );
        assert_eq!(
            sanitized_filename("Hello World", '_', SanitizeMode::Legacy),
            "Hello_World"
        );
        assert_eq!(
            sanitized_filename("Hello.World", '_', SanitizeMode::Legacy),
            "Hello.World"
        );
        assert_eq!(
            sanitized_filename("hello world.wav", '_', SanitizeMode::Legacy),
            "hello_world.wav"
        );
        assert_eq!(
            sanitized_filename("Hello.world.wav", '_', SanitizeMode::Legacy),
            "Hello_world.wav"
        );
        assert_eq!(
            sanitized_filename("hello? + world.wav", '_', SanitizeMode::Legacy),
            "hello_+_world.wav"
        );
        assert_eq!(
            sanitized_filename(
                "Bart_banner_14_5_×_2_5_in.png",
                '_',
                SanitizeMode::Legacy
            ),
            "Bart_banner_14_5_x_2_5_in.png"
        );
        assert_eq!(
            sanitized_filename(
                "hello? &&*()#@+ world.wav",
                '_',
                SanitizeMode::Legacy
            ),
            "hello_@+_world.wav"
        );
        assert_eq!(
            sanitized_filename(
                "August Gold Q&A Audio.m4a.wav",
                '_',
                SanitizeMode::Legacy
            ),
            "August_Gold_Q_A_Audio_m4a.wav"
        );
        assert_eq!(
            sanitized_filename(
                "nested/dir/file name.txt",
                '_',
                SanitizeMode::Legacy
            ),
            "nested/dir/file_name.txt"
        );
        assert_eq!(
            sanitized_filename(
                "/absolute/path/Hello World.txt",
                '_',
                SanitizeMode::Legacy
            ),
            "/absolute/path/Hello_World.txt"
        );
        assert_eq!(
            sanitized_filename(
            "relative/./path/Hello World.txt",
            '_',
            SanitizeMode::Legacy
        ),
        "relative/./path/Hello_World.txt"
    );
}

    #[test]
    fn sanitized_trims_edge_replacements() {
        assert_eq!(
            sanitized_filename(
                "🐾_The_Adventures_of_Marshal_Poppy_The_Great_Sarsaparilla_Heist.md",
                '_',
                SanitizeMode::Full
            ),
            "The_Adventures_of_Marshal_Poppy_The_Great_Sarsaparilla_Heist.md"
        );

        assert_eq!(
            sanitized_filename("  spaced  ", '_', SanitizeMode::Legacy),
            "spaced"
        );

        assert_eq!(
            sanitized_filename(
                "_The_Adventures_of_Marshal_Poppy_The_Great_Sarsaparilla_Heist_.md",
                '_',
                SanitizeMode::Legacy
            ),
            "The_Adventures_of_Marshal_Poppy_The_Great_Sarsaparilla_Heist.md"
        );

        assert_eq!(
            sanitized_filename(
                "nested/  spaced  .txt",
                '_',
                SanitizeMode::Legacy
            ),
            "nested/spaced.txt"
        );
    }

    #[test]
    fn custom_replacement() {
        assert_eq!(
            sanitized_filename(
                "Hello World.txt",
                '-',
                SanitizeMode::Legacy
            ),
            "Hello-World.txt"
        );
    }

    #[test]
    fn full_sanitize_outputs_only_whitelisted_chars() {
        let input = "Hello World! @#[](){}=+,.×é";
        let output =
            sanitized_filename(input, '_', SanitizeMode::Full);
        let path = Path::new(&output);
        let fname = path.file_name().unwrap().to_string_lossy();
        let base = fname.split('.').next().unwrap();
        for ch in base.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '_' || ch == '-',
                "found disallowed character {:?} in output {:?}",
                ch,
                output
            );
        }
    }

    #[test]
    fn cli_replacement_option() {
        let args = vec!["--replacement".to_string(), "-".to_string()];
        let cfg = parse_args(&args).expect("parse_args failed");
        assert_eq!(cfg.replacement, '-');
        assert!(cfg.targets.is_empty());
        assert!(!cfg.full_sanitize);
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
            sanitize_directory_tree(&root, false, '_', SanitizeMode::Legacy)
                .unwrap();

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
            sanitize_directory_tree(&root, false, '-', SanitizeMode::Legacy)
                .unwrap();

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
            sanitize_directory_tree(&root, false, '_', SanitizeMode::Legacy)
                .unwrap();

        let expected_root = PathBuf::from(sanitized_filename(
            root.to_str().unwrap(),
            '_',
            SanitizeMode::Legacy,
        ));
        let expected_child_one = PathBuf::from(sanitized_filename(
            expected_root.join("Child One").to_str().unwrap(),
            '_',
            SanitizeMode::Legacy,
        ));
        let expected_child_two = PathBuf::from(sanitized_filename(
            expected_root.join("Second & Child").to_str().unwrap(),
            '_',
            SanitizeMode::Legacy,
        ));
        let expected_grand_one = PathBuf::from(sanitized_filename(
            expected_child_one.join("Grand Child(1)").to_str().unwrap(),
            '_',
            SanitizeMode::Legacy,
        ));
        let expected_grand_two = PathBuf::from(sanitized_filename(
            expected_child_two.join("Grand (Final)").to_str().unwrap(),
            '_',
            SanitizeMode::Legacy,
        ));

        let expected_files = [
            PathBuf::from(sanitized_filename(
                expected_root.join("Root File?.txt").to_str().unwrap(),
                '_',
                SanitizeMode::Legacy,
            )),
            PathBuf::from(sanitized_filename(
                expected_child_one.join("Clip (A).mov").to_str().unwrap(),
                '_',
                SanitizeMode::Legacy,
            )),
            PathBuf::from(sanitized_filename(
                expected_child_one.join("Clip (B).mov").to_str().unwrap(),
                '_',
                SanitizeMode::Legacy,
            )),
            PathBuf::from(sanitized_filename(
                expected_grand_one.join("Take #1.wav").to_str().unwrap(),
                '_',
                SanitizeMode::Legacy,
            )),
            PathBuf::from(sanitized_filename(
                expected_child_two.join("Audio (Draft).wav").to_str().unwrap(),
                '_',
                SanitizeMode::Legacy,
            )),
            PathBuf::from(sanitized_filename(
                expected_grand_two.join("Mix #2?.wav").to_str().unwrap(),
                '_',
                SanitizeMode::Legacy,
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
    fn sanitize_directory_tree_handles_nonexistent_root() {
        let tmp = temp_dir();
        let missing = tmp.join("does_not_exist");

        let result =
            sanitize_directory_tree(&missing, false, '_', SanitizeMode::Legacy)
                .unwrap();
        assert_eq!(result, missing);
        assert!(!missing.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn sanitize_directory_tree_sanitizes_single_file() {
        let tmp = temp_dir();
        let file = tmp.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let result =
            sanitize_directory_tree(&file, false, '_', SanitizeMode::Legacy)
                .unwrap();
        let expected = tmp.join("file_name.txt");

        assert_eq!(result, expected);
        assert!(!file.exists());
        assert!(expected.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn dry_run_does_not_rename() {
        let tmp = temp_dir();
        let file = tmp.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let desired = PathBuf::from(sanitized_filename(
            file.to_str().unwrap(),
            '_',
            SanitizeMode::Legacy,
        ));
        let result = rename_path(&file, &desired, true).unwrap();

        assert_eq!(result, desired);
        assert!(file.exists());
        assert!(!desired.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn rename_path_noop_when_old_equals_new() {
        let tmp = temp_dir();
        let path = tmp.join("same.txt");
        fs::write(&path, "test").unwrap();

        let result = rename_path(&path, &path, false).unwrap();

        assert_eq!(result, path);
        assert!(path.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn rename_path_skips_when_old_missing() {
        let tmp = temp_dir();
        let old = tmp.join("missing.txt");
        let new_path = tmp.join("new.txt");

        let result = rename_path(&old, &new_path, false).unwrap();

        assert_eq!(result, old);
        assert!(!old.exists());
        assert!(!new_path.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn rename_path_skips_when_new_exists() {
        let tmp = temp_dir();
        let old = tmp.join("old.txt");
        let new_path = tmp.join("new.txt");

        fs::write(&old, "test").unwrap();
        fs::write(&new_path, "other").unwrap();

        let result = rename_path(&old, &new_path, false).unwrap();

        assert_eq!(result, old);
        assert!(old.exists());
        assert!(new_path.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn rename_path_renames_when_possible() {
        let tmp = temp_dir();
        let old = tmp.join("old name.txt");
        let new_path = tmp.join("new_name.txt");

        fs::write(&old, "test").unwrap();

        let result = rename_path(&old, &new_path, false).unwrap();

        assert_eq!(result, new_path);
        assert!(!old.exists());
        assert!(new_path.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn run_non_recursive_renames_target_files() {
        let tmp = temp_dir();
        let file = tmp.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let original = file.to_str().unwrap().to_string();
        let config = Config {
            recursive: false,
            dry_run: false,
            replacement: '_',
            targets: vec![original.clone()],
            full_sanitize: false,
        };

        run(config).unwrap();

        let expected_path = PathBuf::from(sanitized_filename(
            &original,
            '_',
            SanitizeMode::Legacy,
        ));
        assert!(!file.exists());
        assert!(expected_path.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn run_non_recursive_does_not_recurse_into_directories() {
        let tmp = temp_dir();
        let root = tmp.join("dir one");
        let sub = root.join("sub dir");
        fs::create_dir_all(&sub).unwrap();
        let file = sub.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let root_str = root.to_str().unwrap().to_string();
        let config = Config {
            recursive: false,
            dry_run: false,
            replacement: '_',
            targets: vec![root_str.clone()],
            full_sanitize: false,
        };

        run(config).unwrap();

        let expected_root = PathBuf::from(sanitized_filename(
            &root_str,
            '_',
            SanitizeMode::Legacy,
        ));
        assert!(!root.exists());
        assert!(expected_root.exists());

        let expected_sub = expected_root.join("sub dir");
        let expected_file = expected_sub.join("file name.txt");
        assert!(
            expected_sub.is_dir(),
            "expected non-recursive run to leave subdirectory name unchanged"
        );
        assert!(
            expected_file.is_file(),
            "expected non-recursive run to leave file name unchanged"
        );

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn run_recursive_respects_dry_run() {
        let tmp = temp_dir();
        let root = tmp.join("dir one");
        let sub = root.join("sub dir");
        fs::create_dir_all(&sub).unwrap();
        let file = sub.join("file name.txt");
        fs::write(&file, "test").unwrap();

        let root_str = root.to_str().unwrap().to_string();
        let config = Config {
            recursive: true,
            dry_run: true,
            replacement: '_',
            targets: vec![root_str.clone()],
            full_sanitize: false,
        };

        run(config).unwrap();

        assert!(root.exists());
        assert!(sub.exists());
        assert!(file.exists());

        let expected_root =
            PathBuf::from(sanitized_filename(&root_str, '_', SanitizeMode::Legacy));
        assert!(!expected_root.exists());

        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn run_with_args_help_returns_zero() {
        let args = vec!["--help".to_string()];
        let code = run_with_args(&args);
        assert_eq!(code, 0);
    }

    #[test]
    fn run_with_args_reports_missing_targets() {
        let args: Vec<String> = Vec::new();
        let code = run_with_args(&args);
        assert_eq!(code, 1);
    }

    #[test]
    fn run_with_args_propagates_parse_error() {
        let args = vec!["--unknown".to_string()];
        let code = run_with_args(&args);
        assert_eq!(code, 1);
    }

    #[test]
    fn run_with_args_successfully_sanitizes_file() {
        let tmp = temp_dir();
        let file = tmp.join("file name.txt");
        fs::write(&file, "test").unwrap();
        let file_str = file.to_str().unwrap().to_string();
        let args = vec![file_str.clone()];

        let code = run_with_args(&args);
        assert_eq!(code, 0);

        let expected = PathBuf::from(sanitized_filename(
            &file_str,
            '_',
            SanitizeMode::Legacy,
        ));
        assert!(!file.exists());
        assert!(expected.exists());

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
            sanitize_directory_tree(&root, true, '_', SanitizeMode::Legacy)
                .unwrap();
        let expected_root = PathBuf::from(sanitized_filename(
            root.to_str().unwrap(),
            '_',
            SanitizeMode::Legacy,
        ));

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
        assert!(!cfg.full_sanitize);
        assert_eq!(cfg.targets, vec![file_str.clone()]);

        let desired = PathBuf::from(sanitized_filename(
            &file_str,
            cfg.replacement,
            SanitizeMode::Legacy,
        ));
        rename_path(Path::new(&file_str), &desired, cfg.dry_run).unwrap();

        assert!(Path::new(&file_str).exists());
        assert!(!desired.exists());

        fs::remove_dir_all(tmp).unwrap();
    }
}
