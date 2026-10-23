fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(hatchzip::main_from(std::env::args_os()))
}
