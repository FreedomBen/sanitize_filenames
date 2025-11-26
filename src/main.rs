fn main() {
    let code = sanitize_filenames::run_from_env();
    std::process::exit(code);
}

