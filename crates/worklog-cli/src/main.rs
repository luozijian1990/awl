fn main() {
    let mut stdout = std::io::stdout();
    if let Err(err) = worklog_cli::run_with_args(std::env::args(), &mut stdout) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
