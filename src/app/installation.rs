use crate::resources;
use self_update;
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
fn try_replace_binary(source: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    if std::fs::rename(source, dest).is_ok() {
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        if let Ok(false) = std::fs::metadata(dest).map(|m| m.permissions().readonly()) {
            if std::process::Command::new("sudo")
                .args(["cp", &source.to_string_lossy(), &dest.to_string_lossy()])
                .status()
                .map_or(false, |s| s.success())
            {
                let _ = std::fs::remove_file(source);
                return Ok(());
            }
        }
    }
    std::fs::copy(source, dest).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(source);
    Ok(())
}

pub fn spawn_self_update(
    tx: tokio::sync::mpsc::UnboundedSender<crate::app::types::UpdateEvent>,
    target_version: String,
) {
    tokio::spawn(async move {
        let handle = tokio::task::spawn_blocking(move || {
            let relay = "AcoranGonzalezMoray/TraceTUI";
            let parts: Vec<&str> = relay.split('/').collect();
            let user = parts[0];
            let repo = parts[1];

            let releases = self_update::backends::github::ReleaseList::configure()
                .repo_owner(user)
                .repo_name(repo)
                .build()
                .map_err(|e| e.to_string())?
                .fetch()
                .map_err(|e| e.to_string())?;

            let version = target_version.trim_start_matches(['v', 'V']);
            let release = releases
                .iter()
                .find(|r| {
                    let r_v = r.version.trim_start_matches(['v', 'V']);
                    r_v == version
                })
                .ok_or_else(|| format!("Release {} not found", target_version))?;

            let _bin_name = "tracetui";
            let current_target = self_update::get_target();

            let target = if current_target.contains("windows") {
                "x86_64-pc-windows-gnu"
            } else if current_target.contains("linux") {
                "x86_64-unknown-linux-gnu"
            } else {
                current_target
            };

            let asset = release
                .assets
                .iter()
                .find(|a| a.name.contains(target))
                .ok_or_else(|| format!("No asset found for target {}", target))?;

            Ok::<(String, String), String>((asset.download_url.clone(), asset.name.clone()))
        });

        let (url, name) = match handle.await {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => {
                let _ = tx.send(crate::app::types::UpdateEvent::Finished(false, e));
                return;
            }
            Err(e) => {
                let _ = tx.send(crate::app::types::UpdateEvent::Finished(
                    false,
                    e.to_string(),
                ));
                return;
            }
        };

        let client = reqwest::Client::builder()
            .user_agent("TraceTUI-Updater")
            .build()
            .unwrap_or_default();

        let response = match client
            .get(&url)
            .header("Accept", "application/octet-stream")
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => res,
            Ok(res) => {
                let _ = tx.send(crate::app::types::UpdateEvent::Finished(
                    false,
                    format!("Download failed: HTTP {}", res.status()),
                ));
                return;
            }
            Err(e) => {
                let _ = tx.send(crate::app::types::UpdateEvent::Finished(
                    false,
                    format!("Network error: {}", e),
                ));
                return;
            }
        };

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();

        use futures_util::StreamExt;
        while let Some(item) = stream.next().await {
            let chunk = match item {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(crate::app::types::UpdateEvent::Finished(
                        false,
                        format!("Stream error: {}", e),
                    ));
                    return;
                }
            };
            downloaded += chunk.len() as u64;
            buffer.extend_from_slice(&chunk);

            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                let _ = tx.send(crate::app::types::UpdateEvent::Progress(progress));
            }
        }

        let status = tokio::task::spawn_blocking(move || {
            let bin_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let tmp_dir = bin_path.parent().unwrap().join(".tracetui_update_tmp");

            std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
            let tmp_path = tmp_dir.join(&name);
            std::fs::write(&tmp_path, buffer).map_err(|e| e.to_string())?;

            let bin_name = if cfg!(target_os = "windows") {
                "tracetui.exe"
            } else {
                "tracetui"
            };
            let extracted_bin = tmp_dir.join(bin_name);

            if name.ends_with(".zip") {
                let file = std::fs::File::open(&tmp_path).map_err(|e| e.to_string())?;
                let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
                    if file.name() == bin_name {
                        let mut out =
                            std::fs::File::create(&extracted_bin).map_err(|e| e.to_string())?;
                        std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
                        break;
                    }
                }
            } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
                let tar_gz = std::fs::File::open(&tmp_path).map_err(|e| e.to_string())?;
                let tar = flate2::read::GzDecoder::new(tar_gz);
                let mut archive = tar::Archive::new(tar);
                for entry in archive.entries().map_err(|e| e.to_string())? {
                    let mut entry = entry.map_err(|e| e.to_string())?;
                    let path = entry.path().map_err(|e| e.to_string())?;
                    if path.to_str() == Some(bin_name) {
                        entry.unpack(&extracted_bin).map_err(|e| e.to_string())?;
                        break;
                    }
                }
            } else {
                std::fs::rename(&tmp_path, &extracted_bin).map_err(|e| e.to_string())?;
            }

            if !extracted_bin.exists() {
                return Err(format!(
                    "Could not find binary {} in the update package",
                    bin_name
                ));
            }

            #[cfg(windows)]
            {
                // Windows: rename running exe -> .old (works while process is running),
                // then place the new binary in its place
                let old_path = bin_path.with_extension("exe.old");
                let _ = std::fs::rename(&bin_path, &old_path);
                try_replace_binary(&extracted_bin, &bin_path)?;
            }
            #[cfg(not(windows))]
            {
                // Linux/macOS: try direct, fallback to sudo if permission denied
                try_replace_binary(&extracted_bin, &bin_path)?;
            }

            let _ = std::fs::remove_dir_all(&tmp_dir);
            Ok::<(), String>(())
        })
        .await;

        match status {
            Ok(Ok(_)) => {
                let _ = tx.send(crate::app::types::UpdateEvent::Finished(
                    true,
                    "Update successful! Restart the application to apply changes.".to_string(),
                ));
            }
            Ok(Err(e)) => {
                let _ = tx.send(crate::app::types::UpdateEvent::Finished(
                    false,
                    format!("Update process failed: {}", e),
                ));
            }
            Err(e) => {
                let _ = tx.send(crate::app::types::UpdateEvent::Finished(
                    false,
                    format!("Update task panicked: {}", e),
                ));
            }
        }
    });
}
