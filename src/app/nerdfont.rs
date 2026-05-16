pub fn has_nerdfont() -> bool {
    #[cfg(target_os = "windows")]
    {
        has_nerdfont_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        has_nerdfont_unix()
    }
}
#[cfg(target_os = "windows")]
fn has_nerdfont_windows() -> bool {
    let script = r#"
$found = $false
$paths = @(
    "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
    "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Fonts"
)
foreach ($p in $paths) {
    $reg = Get-ItemProperty $p -ErrorAction SilentlyContinue
    if ($reg -and ($reg.PSObject.Properties | Where-Object { $_.Name -match '(?i)Nerd| NF[ )]| NF$' })) {
        $found = $true; break
    }
}
if (-not $found) {
    $dirs = @("$env:WINDIR\Fonts", "$env:LOCALAPPDATA\Microsoft\Windows\Fonts")
    foreach ($d in $dirs) {
        if (Test-Path $d) {
            $files = Get-ChildItem $d -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '(?i)Nerd| NF[ )-]| NF$' }
            if ($files) { $found = $true; break }
        }
    }
}
if ($found) { exit 0 } else { exit 1 }
"#;
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();
    output.map(|o| o.status.success()).unwrap_or(false)
}
#[cfg(not(target_os = "windows"))]
fn has_nerdfont_unix() -> bool {
    std::process::Command::new("fc-list")
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .to_lowercase()
                .contains("nerd")
        })
        .unwrap_or(false)
}
