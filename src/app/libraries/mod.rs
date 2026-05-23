use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryRisk {
    Safe,
    Suspicious,
    Critical,
    Unknown,
}

impl LibraryRisk {
    pub fn as_str(&self) -> &'static str {
        match self {
            LibraryRisk::Safe => "Safe",
            LibraryRisk::Suspicious => "Suspicious",
            LibraryRisk::Critical => "Critical",
            LibraryRisk::Unknown => "Unknown",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Safe" => LibraryRisk::Safe,
            "Suspicious" => LibraryRisk::Suspicious,
            "Critical" => LibraryRisk::Critical,
            _ => LibraryRisk::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureStatus {
    Signed,
    Unsigned,
    Invalid,
    Unknown,
}

impl SignatureStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureStatus::Signed => "Signed",
            SignatureStatus::Unsigned => "Unsigned",
            SignatureStatus::Invalid => "Invalid",
            SignatureStatus::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryOrigin {
    System,
    ProgramFiles,
    UserSpace,
    Temp,
    Unknown,
}

impl LibraryOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            LibraryOrigin::System => "System",
            LibraryOrigin::ProgramFiles => "Program",
            LibraryOrigin::UserSpace => "UserSpace",
            LibraryOrigin::Temp => "Temp",
            LibraryOrigin::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryInfo {
    pub name: String,
    pub path: String,
    pub pid: u32,
    pub process_name: String,
    pub signature: SignatureStatus,
    pub origin: LibraryOrigin,
    pub size: u64,
    pub risk: String,

    pub sha256: String,

    pub is_signed: Option<bool>,
}

pub fn inspect_libraries(
    processes: &[crate::app::process::ProcessInfo],
    app_connections: &[crate::app::types::AppConnection],
) -> Vec<LibraryInfo> {
    let mut result = Vec::new();
    let pid_set: std::collections::HashSet<u32> = app_connections.iter().map(|a| a.pid).collect();
    let pid_to_name: HashMap<u32, &str> = app_connections
        .iter()
        .map(|a| (a.pid, a.process_name.as_str()))
        .collect();

    for process in processes {
        if !pid_set.contains(&process.pid) {
            continue;
        }
        let libs = get_libraries_for_pid(process.pid);
        let pname = pid_to_name
            .get(&process.pid)
            .copied()
            .unwrap_or(&process.name);
        for mut lib in libs {
            lib.origin = classify_origin(&lib.path);
            lib.risk = classify_risk(&lib.path, &lib.origin);
            lib.signature = check_signature(&lib.path);
            lib.is_signed = match lib.signature {
                SignatureStatus::Signed => Some(true),
                SignatureStatus::Unsigned | SignatureStatus::Invalid => Some(false),
                SignatureStatus::Unknown => None,
            };
            lib.process_name = pname.to_string();

            if lib.risk != "Safe" {
                lib.sha256 = compute_sha256_partial(&lib.path);
            }
            result.push(lib);
        }
    }

    result.sort_by(|a, b| {
        risk_sort_key(&b.risk)
            .cmp(&risk_sort_key(&a.risk))
            .then_with(|| b.pid.cmp(&a.pid))
            .then_with(|| b.size.cmp(&a.size))
    });
    result
}

fn risk_sort_key(r: &str) -> u8 {
    match r {
        "Critical" => 3,
        "Suspicious" => 2,
        "Unknown" => 1,
        _ => 0,
    }
}

fn get_libraries_for_pid(pid: u32) -> Vec<LibraryInfo> {
    #[cfg(target_os = "windows")]
    {
        get_libraries_windows(pid)
    }
    #[cfg(target_os = "linux")]
    {
        get_libraries_linux(pid)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = pid;
        Vec::new()
    }
}

#[cfg(target_os = "windows")]
fn get_libraries_windows(pid: u32) -> Vec<LibraryInfo> {
    let script = format!(
        "$ErrorActionPreference = 'Stop'; \
         try {{ \
             $p = Get-Process -Id {pid} -ErrorAction Stop; \
             $p.Modules | ForEach-Object {{ \
                 $m = $_; \
                 $s = if ($m.FileName -and (Test-Path $m.FileName)) {{ \
                     (Get-Item $m.FileName).Length \
                 }} else {{ 0 }}; \
                 \"{pid}|$($m.ModuleName)|$($m.FileName)|$s\" \
             }} \
         }} catch {{ }}",
        pid = pid
    );
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output();
    let mut libs = Vec::new();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                if parts.len() == 4 {
                    let name = parts[1].trim().to_string();
                    let path = parts[2].trim().to_string();
                    let size = parts[3].trim().parse::<u64>().unwrap_or(0);
                    if !name.is_empty() {
                        libs.push(LibraryInfo {
                            pid,
                            process_name: String::new(),
                            signature: SignatureStatus::Unknown,
                            origin: LibraryOrigin::Unknown,
                            is_signed: None,
                            risk: String::new(),
                            sha256: String::new(),
                            name,
                            path,
                            size,
                        });
                    }
                }
            }
        }
    }
    libs
}

#[cfg(target_os = "linux")]
fn get_libraries_linux(pid: u32) -> Vec<LibraryInfo> {
    let maps_path = format!("/proc/{}/maps", pid);
    let content = std::fs::read_to_string(&maps_path);
    let mut seen = std::collections::HashSet::new();
    let mut libs = Vec::new();
    if let Ok(data) = content {
        for line in data.lines() {
            if let Some(pos) = line.find(" /") {
                let path = line[pos + 1..].trim().to_string();
                if (path.ends_with(".so") || path.contains(".so.")) && !seen.contains(&path) {
                    seen.insert(path.clone());
                    let name = std::path::Path::new(&path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    libs.push(LibraryInfo {
                        pid,
                        process_name: String::new(),
                        name,
                        path,
                        size,
                        signature: SignatureStatus::Unknown,
                        origin: LibraryOrigin::Unknown,
                        is_signed: None,
                        risk: String::new(),
                        sha256: String::new(),
                    });
                }
            }
        }
    }
    libs
}

pub fn classify_origin(path: &str) -> LibraryOrigin {
    let lower = path.to_lowercase();

    let temp_dirs = [
        "\\temp\\",
        "\\tmp\\",
        "/tmp/",
        "/dev/shm/",
        "/var/tmp/",
        "\\appdata\\local\\temp\\",
    ];
    for d in &temp_dirs {
        if lower.contains(d) {
            return LibraryOrigin::Temp;
        }
    }

    let sys_dirs = [
        "\\system32\\",
        "\\syswow64\\",
        "\\windows\\",
        "/lib/",
        "/lib64/",
        "/usr/lib/",
        "/usr/lib64/",
        "/lib/x86_64-linux-gnu/",
        "/lib/aarch64-linux-gnu/",
        "/usr/share/",
        "/boot/",
    ];
    for d in &sys_dirs {
        if lower.contains(d) {
            return LibraryOrigin::System;
        }
    }

    let prog_dirs = [
        "\\program files\\",
        "\\program files (x86)\\",
        "/opt/",
        "/usr/local/",
    ];
    for d in &prog_dirs {
        if lower.contains(d) {
            return LibraryOrigin::ProgramFiles;
        }
    }

    let user_dirs = [
        "\\users\\",
        "\\appdata\\",
        "\\downloads\\",
        "\\desktop\\",
        "/home/",
        "/root/",
    ];
    for d in &user_dirs {
        if lower.contains(d) {
            return LibraryOrigin::UserSpace;
        }
    }

    LibraryOrigin::Unknown
}

pub fn classify_risk(path: &str, origin: &LibraryOrigin) -> String {
    let lower = path.to_lowercase();

    let critical_names = [
        "inject",
        "hook",
        "hijack",
        "payload",
        "shellcode",
        "keylog",
        "rootkit",
        "backdoor",
        "meterpreter",
        "cobalt",
        "mimikatz",
        "lsadump",
    ];
    let fname = std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    for kw in &critical_names {
        if fname.contains(kw) {
            return "Critical".to_string();
        }
    }

    if *origin == LibraryOrigin::Temp {
        return "Critical".to_string();
    }

    let suspicious_dirs = [
        "\\appdata\\roaming\\",
        "\\downloads\\",
        "\\desktop\\",
        "\\users\\",
        "/home/",
        "/root/",
    ];
    for d in &suspicious_dirs {
        if lower.contains(d) {
            return "Suspicious".to_string();
        }
    }

    "Safe".to_string()
}

pub fn check_signature(path: &str) -> SignatureStatus {
    if path.is_empty() {
        return SignatureStatus::Unknown;
    }

    #[cfg(target_os = "windows")]
    {
        check_signature_windows(path)
    }
    #[cfg(target_os = "linux")]
    {
        check_signature_linux(path)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = path;
        SignatureStatus::Unknown
    }
}

#[cfg(target_os = "windows")]
fn check_signature_windows(path: &str) -> SignatureStatus {
    let escaped = path.replace('\'', "''");
    let script = format!("(Get-AuthenticodeSignature '{}').Status", escaped);
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            match s.as_str() {
                "valid" => SignatureStatus::Signed,
                "notsinged" | "notsigned" => SignatureStatus::Unsigned,
                "hashmismatch" | "nottrustprovider" | "unknownerror" => SignatureStatus::Invalid,
                _ => SignatureStatus::Unknown,
            }
        }
        _ => SignatureStatus::Unknown,
    }
}

#[cfg(target_os = "linux")]
fn check_signature_linux(path: &str) -> SignatureStatus {
    if is_package_managed(path) {
        return SignatureStatus::Signed;
    }

    let output = std::process::Command::new("gpg")
        .args(["--verify", path])
        .output();
    match output {
        Ok(out) if out.status.success() => SignatureStatus::Signed,
        Ok(_) => SignatureStatus::Unsigned,
        Err(_) => {
            if is_system_path(path) {
                SignatureStatus::Signed
            } else {
                SignatureStatus::Unsigned
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn is_package_managed(path: &str) -> bool {
    if let Ok(out) = std::process::Command::new("dpkg")
        .args(["-S", path])
        .output()
    {
        if out.status.success() {
            return true;
        }
    }

    if let Ok(out) = std::process::Command::new("rpm")
        .args(["-qf", path])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            if !s.contains("not owned") {
                return true;
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn is_system_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    let system_prefixes = [
        "/lib/",
        "/lib64/",
        "/usr/lib/",
        "/usr/lib64/",
        "/usr/share/",
        "/boot/",
    ];
    system_prefixes.iter().any(|p| lower.starts_with(p))
}

pub fn compute_sha256_partial(path: &str) -> String {
    use std::io::Read;
    let file = std::fs::File::open(path);
    match file {
        Ok(mut f) => {
            let mut buf = vec![0u8; 65536];
            let n = f.read(&mut buf).unwrap_or(0);
            if n == 0 {
                return String::new();
            }
            sha256_hex(&buf[..n])
        }
        Err(_) => String::new(),
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}(fnv64)", hash)
}

pub fn export_libraries_json(libs: &[LibraryInfo]) -> Result<String, String> {
    serde_json::to_string_pretty(libs).map_err(|e| e.to_string())
}

pub fn export_libraries_csv(libs: &[LibraryInfo]) -> String {
    let mut out = String::from("pid,process_name,name,path,size,risk,origin,signature,sha256\n");
    for l in libs {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            l.pid,
            csv_escape(&l.process_name),
            csv_escape(&l.name),
            csv_escape(&l.path),
            l.size,
            l.risk,
            l.origin.as_str(),
            l.signature.as_str(),
            l.sha256,
        ));
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

impl crate::app::App {
    pub fn process_libraries_results(&mut self) {
        if let Some(ref rx) = self.libraries_rx {
            if let Ok(libs) = rx.try_recv() {
                self.libraries = libs;
                self.libraries_loading = false;
                self.libraries_loaded_once = true;
                self.libraries_rx = None;
            }
            return;
        }

        if self.libraries_loading && !self.processes.is_empty() {
            self.libraries_loading = false;
            self.refresh_libraries();
        }
    }

    pub fn group_libs_by_process(&self) -> Vec<(String, usize)> {
        let mut map: HashMap<String, usize> = HashMap::new();
        for lib in &self.libraries {
            *map.entry(lib.process_name.clone()).or_insert(0) += 1;
        }
        let mut result: Vec<(String, usize)> = map.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    pub fn get_libs_for_selected_process_owned(
        &self,
        search_query: &str,
        risk_filter: Option<&str>,
    ) -> Vec<LibraryInfo> {
        let groups = self.group_libs_by_process();
        let pname = groups
            .get(self.selected_library_process_index)
            .map(|(n, _)| n.as_str())
            .unwrap_or("");
        let sq = search_query.to_lowercase();
        self.libraries
            .iter()
            .filter(|l| {
                l.process_name == pname
                    && (sq.is_empty()
                        || l.name.to_lowercase().contains(&sq)
                        || l.path.to_lowercase().contains(&sq))
                    && match risk_filter {
                        Some(f) => l.risk == f,
                        None => true,
                    }
            })
            .cloned()
            .collect()
    }

    pub fn rehash_suspicious_libraries(&mut self) {
        for lib in self.libraries.iter_mut() {
            if lib.risk != "Safe" && lib.sha256.is_empty() {
                lib.sha256 = compute_sha256_partial(&lib.path);
            }
        }
    }

    pub fn export_libraries_with_filter(
        &mut self,
        fmt: &str,
        search_query: &str,
        risk_filter: Option<&str>,
    ) {
        use std::io::Write;

        let libs = self.get_libs_for_selected_process_owned(search_query, risk_filter);
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

        let (default_name, content) = if fmt == "csv" {
            (
                format!("libraries_{}.csv", timestamp),
                export_libraries_csv(&libs),
            )
        } else {
            (
                format!("libraries_{}.json", timestamp),
                match export_libraries_json(&libs) {
                    Ok(s) => s,
                    Err(e) => {
                        self.status_message = format!("Export error: {}", e);
                        return;
                    }
                },
            )
        };

        let path = {
            #[cfg(target_os = "windows")]
            {
                pick_save_path_windows(&default_name)
                    .unwrap_or_else(|| std::path::PathBuf::from(&default_name))
            }
            #[cfg(target_os = "linux")]
            {
                pick_save_path_linux(&default_name)
                    .unwrap_or_else(|| std::path::PathBuf::from(&default_name))
            }
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            {
                std::path::PathBuf::from(&default_name)
            }
        };

        match std::fs::File::create(&path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(content.as_bytes()) {
                    self.status_message = format!("Write error: {}", e);
                } else {
                    self.status_message =
                        format!("Exported {} libs → {}", libs.len(), path.display());
                }
            }
            Err(e) => {
                self.status_message = format!("Create error: {}", e);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn pick_save_path_windows(default_name: &str) -> Option<std::path::PathBuf> {
    let script = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.SaveFileDialog; $f.FileName = '{}'; if ($f.ShowDialog() -eq 'OK') {{ $f.FileName }}"#,
        default_name
    );
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(path))
    }
}

#[cfg(target_os = "linux")]
fn pick_save_path_linux(default_name: &str) -> Option<std::path::PathBuf> {
    let output = std::process::Command::new("zenity")
        .args([
            "--file-selection",
            "--save",
            "--title=Export Libraries",
            &format!("--filename={}", default_name),
        ])
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(path))
    }
}
