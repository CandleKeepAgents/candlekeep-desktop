use std::process::Command;

/// Create a platform-appropriate shell command.
/// On Windows, uses `cmd.exe /C`; on Unix, uses `sh -c`.
pub fn shell_command(cmd: &str, path_env: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut c = Command::new("cmd.exe");
        c.args(["/C", cmd]).env("PATH", path_env);
        c
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd).env("PATH", path_env);
        c
    }
}
