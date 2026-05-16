use crate::resources;
pub fn make_failed_output(msg: &[u8]) -> std::process::Output {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg("exit 1")
        .status()
        .unwrap_or_else(|_| {
            std::process::Command::new("cmd.exe")
                .arg("/c")
                .arg("exit 1")
                .status()
                .expect("Cannot create failure ExitStatus")
        });
    std::process::Output {
        status,
        stdout: Vec::new(),
        stderr: msg.to_vec(),
    }
}
pub fn spawn_check_sudo(child: &mut Option<tokio::sync::oneshot::Receiver<std::process::Output>>) {
    let (tx, rx) = tokio::sync::oneshot::channel();
    *child = Some(rx);
    tokio::spawn(async move {
        let output = std::process::Command::new("sudo")
            .args(["-n", "apt", "install", "-y", "net-tools"])
            .output()
            .unwrap_or_else(|_| make_failed_output(b"Failed to execute sudo"));
        let _ = tx.send(output);
    });
}
pub fn spawn_install_with_password(
    child: &mut Option<tokio::sync::oneshot::Receiver<std::process::Output>>,
    password: String,
) {
    let (tx, rx) = tokio::sync::oneshot::channel();
    *child = Some(rx);
    tokio::spawn(async move {
        let mut child = match std::process::Command::new("sudo")
            .args(["-S", "apt", "install", "-y", "net-tools"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                let _ = tx.send(make_failed_output(b"Failed to spawn sudo"));
                return;
            }
        };
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = writeln!(stdin, "{}", password);
            let _ = stdin.flush();
        }
        let output = child
            .wait_with_output()
            .unwrap_or_else(|_| make_failed_output(b"Failed to wait for sudo"));
        let _ = tx.send(output);
    });
}
pub fn spawn_nerdfont_install(
    rx: &mut Option<tokio::sync::oneshot::Receiver<String>>,
    message: &mut String,
) {
    let (tx, r) = tokio::sync::oneshot::channel();
    *rx = Some(r);
    *message = "Downloading JetBrains Mono Nerd Font...".to_string();
    tokio::spawn(async move {
        let result = run_nerdfont_script();
        let msg = match &result {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if stdout.contains("OK") {
                    "Installed! Restart your terminal and select\nJetBrainsMono Nerd Font in the settings.".to_string()
                } else {
                    format!("Installed, but verification uncertain:\n{}", stdout.trim())
                }
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let stdout = String::from_utf8_lossy(&o.stdout);
                format!("Installation failed.\n{}\n{}", stdout.trim(), stderr.trim())
            }
            Err(e) => format!("Failed to run installer:\n{}", e),
        };
        let _ = tx.send(msg);
    });
}
#[cfg(target_os = "windows")]
fn run_nerdfont_script() -> std::io::Result<std::process::Output> {
    let url = &resources::URLS.nerd_font_repo_url;
    let script = format!(
        r#"
$zip = "$env:TEMP\JetBrainsMono.zip"
$dest = "$env:LOCALAPPDATA\Microsoft\Windows\Fonts\JetBrainsMonoNerd"
try {{
    Write-Output "Downloading..."
    Invoke-WebRequest -Uri "{url}" -OutFile $zip -UseBasicParsing
    Write-Output "Extracting..."
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    Expand-Archive -Path $zip -DestinationPath $dest -Force
    Remove-Item $zip -Force
    Write-Output "OK"
}} catch {{
    Write-Output "FAIL: $_"
}}
"#
    );
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
}
#[cfg(target_os = "linux")]
fn run_nerdfont_script() -> std::io::Result<std::process::Output> {
    let url = &resources::URLS.nerd_font_repo_url;
    let script = format!(
        "mkdir -p $HOME/.local/share/fonts/JetBrainsMonoNerd && \
         curl -L --connect-timeout 30 -o /tmp/JetBrainsMono.zip \
         '{url}' && \
         python3 -c \"import zipfile,os;d=os.path.expanduser('$HOME/.local/share/fonts/JetBrainsMonoNerd');os.makedirs(d,exist_ok=True);zipfile.ZipFile('/tmp/JetBrainsMono.zip').extractall(d)\" && \
         rm -f /tmp/JetBrainsMono.zip && \
         (command -v fc-cache >/dev/null 2>&1 && fc-cache -fv || true) && \
         echo OK"
    );
    std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .output()
}
