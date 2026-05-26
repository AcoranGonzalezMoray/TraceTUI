use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const LIBRARY_ACTION_COUNT: usize = 7;

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

pub fn inspect_libraries_batched(
    processes: &[crate::app::process::ProcessInfo],
    app_connections: &[crate::app::types::AppConnection],
    tx: std::sync::mpsc::Sender<Vec<LibraryInfo>>,
) {
    let pid_set: std::collections::HashSet<u32> = app_connections.iter().map(|a| a.pid).collect();
    let pid_to_name: std::collections::HashMap<u32, &str> = app_connections
        .iter()
        .map(|a| (a.pid, a.process_name.as_str()))
        .collect();

    let valid_pids: Vec<u32> = processes
        .iter()
        .filter(|p| pid_set.contains(&p.pid))
        .map(|p| p.pid)
        .collect();

    let libs = get_libraries_batch(&valid_pids);
    let batch_size = crate::config::LIBRARY_BATCH_SIZE;

    for chunk in libs.chunks(batch_size) {
        let mut batch = Vec::with_capacity(batch_size);
        for mut lib in chunk.iter().cloned() {
            lib.origin = classify_origin(&lib.path);
            lib.risk = classify_risk(&lib.path, &lib.origin);
            lib.signature = check_signature(&lib.path);
            lib.is_signed = match lib.signature {
                SignatureStatus::Signed => Some(true),
                SignatureStatus::Unsigned | SignatureStatus::Invalid => Some(false),
                SignatureStatus::Unknown => None,
            };
            lib.process_name = pid_to_name
                .get(&lib.pid)
                .copied()
                .unwrap_or_default()
                .to_string();

            if lib.risk != "Safe" {
                lib.sha256 = compute_sha256_partial(&lib.path);
            }
            batch.push(lib);
        }
        if tx.send(batch).is_err() {
            return;
        }
    }
}

fn get_libraries_batch(pids: &[u32]) -> Vec<LibraryInfo> {
    #[cfg(target_os = "windows")]
    {
        get_libraries_windows(pids)
    }
    #[cfg(target_os = "linux")]
    {
        let mut result = Vec::new();
        for &pid in pids {
            result.extend(get_libraries_linux(pid));
        }
        result
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = pids;
        Vec::new()
    }
}

pub(crate) fn risk_sort_key(r: &str) -> u8 {
    match r {
        "Critical" => 3,
        "Suspicious" => 2,
        "Unknown" => 1,
        _ => 0,
    }
}

#[cfg(target_os = "windows")]
fn get_libraries_windows(pids: &[u32]) -> Vec<LibraryInfo> {
    if pids.is_empty() {
        return Vec::new();
    }

    let pids_csv = pids
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let script = format!(
        r#"
        $ErrorActionPreference = 'SilentlyContinue';
        $targetPids = @({pids_csv});
        foreach ($procId in $targetPids) {{
            try {{
                $p = Get-Process -Id $procId -ErrorAction Stop;
                $p.Modules | ForEach-Object {{
                    $m = $_;
                    if ($m.FileName -and (Test-Path $m.FileName)) {{
                        $size = (Get-Item $m.FileName).Length;
                        "$procId|$($m.ModuleName)|$($m.FileName)|$size"
                    }}
                }}
            }} catch {{ }}
        }}
        "#,
        pids_csv = pids_csv
    );

    let mut libs = Vec::new();
    if let Ok(out) = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
    {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                if parts.len() == 4 {
                    if let Ok(pid) = parts[0].trim().parse::<u32>() {
                        let name = parts[1].trim().to_string();
                        let path = parts[2].trim().to_string();
                        let size = parts[3].trim().parse::<u64>().unwrap_or(0);
                        if !name.is_empty() && pid > 0 {
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
            let mut buf = vec![0u8; crate::config::SHA256_BUFFER_SIZE];
            let n = f.read(&mut buf).unwrap_or(0);
            if n == 0 {
                return String::new();
            }
            fingerprint_fnv64(&buf[..n])
        }
        Err(_) => String::new(),
    }
}

fn fingerprint_fnv64(data: &[u8]) -> String {
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
        use std::sync::mpsc::TryRecvError;

        if let Some(ref rx) = self.libraries.libraries_rx {
            loop {
                match rx.try_recv() {
                    Ok(libs) => {
                        self.libraries.libraries.extend(libs);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.libraries.libraries.sort_by(|a, b| {
                            crate::app::libraries::risk_sort_key(&b.risk)
                                .cmp(&crate::app::libraries::risk_sort_key(&a.risk))
                                .then_with(|| b.pid.cmp(&a.pid))
                                .then_with(|| b.size.cmp(&a.size))
                        });
                        self.libraries.libraries_loading = false;
                        self.libraries.libraries_loaded_once = true;
                        self.libraries.libraries_rx = None;
                        break;
                    }
                }
            }
        } else if self.libraries.libraries_loading && !self.network.processes.is_empty() {
            self.libraries.libraries_loading = false;
            self.refresh_libraries();
        }
    }

    pub fn group_libs_by_process(&self) -> Vec<(String, usize)> {
        let mut map: HashMap<String, usize> = HashMap::new();
        for lib in &self.libraries.libraries {
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
            .get(self.libraries.selected_library_process_index)
            .map(|(n, _)| n.as_str())
            .unwrap_or("");
        let sq = search_query.to_lowercase();
        self.libraries
            .libraries
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
        for lib in self.libraries.libraries.iter_mut() {
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
                        self.ui.status_message = format!("Export error: {}", e);
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
                    self.ui.status_message = format!("Write error: {}", e);
                } else {
                    self.ui.status_message =
                        format!("Exported {} libs → {}", libs.len(), path.display());
                }
            }
            Err(e) => {
                self.ui.status_message = format!("Create error: {}", e);
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

fn read_u16_le(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buf[offset], buf[offset + 1]])
}
fn read_u32_le(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ])
}
fn read_u64_le(buf: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
        buf[offset + 4],
        buf[offset + 5],
        buf[offset + 6],
        buf[offset + 7],
    ])
}

fn find_pe_text_section(data: &[u8]) -> Option<(usize, usize)> {
    if data.len() < 64 || read_u16_le(data, 0) != 0x5A4D {
        return None;
    }
    let pe_offset = read_u32_le(data, 0x3C) as usize;
    if pe_offset + 4 + 20 > data.len() {
        return None;
    }
    if read_u32_le(data, pe_offset) != 0x00004550 {
        return None;
    }
    let num_sections = read_u16_le(data, pe_offset + 4 + 2) as usize;
    let opt_header_size = read_u16_le(data, pe_offset + 4 + 16) as usize;
    let section_off = pe_offset + 4 + 20 + opt_header_size;
    for i in 0..num_sections {
        let off = section_off + i * 40;
        if off + 40 > data.len() {
            break;
        }
        let name = std::str::from_utf8(&data[off..off + 8])
            .unwrap_or("")
            .trim_end_matches('\0');
        if name.eq_ignore_ascii_case(".text") || name.eq_ignore_ascii_case(".textbss") {
            let raw_size = read_u32_le(data, off + 16) as usize;
            let raw_off = read_u32_le(data, off + 20) as usize;
            if raw_off + raw_size <= data.len() {
                return Some((raw_off, raw_size));
            }
        }
    }
    None
}

fn find_elf_text_section(data: &[u8]) -> Option<(usize, usize)> {
    if data.len() < 64 || data[..4] != [0x7f, 0x45, 0x4c, 0x46] {
        return None;
    }
    let is_64bit = data[4] == 2;
    if data[5] != 1 {
        return None;
    }
    let (shoff, shentsize, shnum, shstrndx) = if is_64bit {
        (
            read_u64_le(data, 0x28) as usize,
            read_u16_le(data, 0x3A) as usize,
            read_u16_le(data, 0x3C) as usize,
            read_u16_le(data, 0x3E) as usize,
        )
    } else {
        (
            read_u32_le(data, 0x20) as usize,
            read_u16_le(data, 0x2E) as usize,
            read_u16_le(data, 0x30) as usize,
            read_u16_le(data, 0x32) as usize,
        )
    };
    if shstrndx >= shnum || shoff + shnum * shentsize > data.len() {
        return None;
    }
    let strtab_off = shoff + shstrndx * shentsize;
    let strtab_sh_off = if is_64bit {
        read_u64_le(data, strtab_off + 0x18) as usize
    } else {
        read_u32_le(data, strtab_off + 0x10) as usize
    };
    let strtab_sh_size = if is_64bit {
        read_u64_le(data, strtab_off + 0x20) as usize
    } else {
        read_u32_le(data, strtab_off + 0x14) as usize
    };
    if strtab_sh_off + strtab_sh_size > data.len() {
        return None;
    }
    let strtab = &data[strtab_sh_off..strtab_sh_off + strtab_sh_size];
    for i in 0..shnum {
        let off = shoff + i * shentsize;
        let name_off = read_u32_le(data, off) as usize;
        let name = if name_off < strtab.len() {
            let end = strtab[name_off..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or_default();
            std::str::from_utf8(&strtab[name_off..name_off + end]).unwrap_or("")
        } else {
            ""
        };
        if name == ".text" {
            let (sh_offset, sh_size) = if is_64bit {
                (
                    read_u64_le(data, off + 0x18) as usize,
                    read_u64_le(data, off + 0x20) as usize,
                )
            } else {
                (
                    read_u32_le(data, off + 0x10) as usize,
                    read_u32_le(data, off + 0x14) as usize,
                )
            };
            if sh_offset + sh_size <= data.len() {
                return Some((sh_offset, sh_size));
            }
        }
    }
    None
}

pub fn load_binary_hex(path: &str) -> Vec<String> {
    use std::io::Read;
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return vec![format!("Error: cannot open file: {}", e)],
    };
    let file_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => return vec![format!("Error: cannot read metadata: {}", e)],
    };
    let read_size = file_size.min(crate::config::BINARY_VIEW_MAX_SIZE) as usize;
    let mut data = vec![0u8; read_size];
    if file.read_exact(&mut data).is_err() {
        return vec!["Error: cannot read file".to_string()];
    }
    let mut lines = Vec::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let addr = i * 16;
        let mut hl = String::new();
        let mut hr = String::new();
        let mut asc = String::new();
        for (j, &b) in chunk.iter().enumerate() {
            if j < 8 {
                if j > 0 {
                    hl.push(' ');
                }
                hl.push_str(&format!("{:02x}", b));
            } else {
                if j > 8 {
                    hr.push(' ');
                }
                hr.push_str(&format!("{:02x}", b));
            }
            asc.push(if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            });
        }
        lines.push(format!("{:08X}  {:23}  {:23}  |{}|", addr, hl, hr, asc));
    }
    if file_size > crate::config::BINARY_VIEW_MAX_SIZE {
        lines.push(format!(
            "... (file truncated to {} MB)",
            crate::config::BINARY_VIEW_MAX_SIZE / 1024 / 1024
        ));
    }
    lines
}

pub fn load_binary_disasm(path: &str) -> Vec<String> {
    use std::io::Read;
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return vec![format!("Error: cannot open file: {}", e)],
    };
    let file_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => return vec![format!("Error: cannot read metadata: {}", e)],
    };
    let read_size = file_size.min(crate::config::BINARY_VIEW_MAX_SIZE) as usize;
    let mut data = vec![0u8; read_size];
    if file.read_exact(&mut data).is_err() {
        return vec!["Error: cannot read file".to_string()];
    }
    let is_pe = data.starts_with(&[0x4D, 0x5A]);
    let is_elf = data.starts_with(&[0x7f, 0x45, 0x4c, 0x46]);
    let section = if is_pe {
        find_pe_text_section(&data)
    } else if is_elf {
        find_elf_text_section(&data)
    } else {
        Some((0, read_size))
    };
    let (offset, size) = match section {
        Some(s) => s,
        None => return vec!["No executable code section found".to_string()],
    };
    let code_size = size.min(crate::config::DISASM_MAX_BYTES);
    let code = &data[offset..offset + code_size];
    let bitness: u32 = if is_pe {
        let pe_off = read_u32_le(&data, 0x3C) as usize;
        if pe_off + 4 + 20 + 2 <= data.len() && read_u16_le(&data, pe_off + 4 + 20) == 0x020B {
            64
        } else {
            32
        }
    } else if is_elf && data[4] == 2 {
        64
    } else if is_elf {
        32
    } else {
        64
    };
    let mut lines = Vec::new();
    lines.push(format!("; Architecture: {}-bit", bitness));
    lines.push(format!(
        "; Section offset: 0x{:X}, size: {} bytes",
        offset, size
    ));
    lines.push(format!("; Displaying {} bytes", code.len()));
    lines.push(String::new());
    let mut decoder =
        iced_x86::Decoder::with_ip(bitness, code, offset as u64, iced_x86::DecoderOptions::NONE);
    let mut formatter = iced_x86::IntelFormatter::new();
    let mut output = String::new();
    let mut instr = iced_x86::Instruction::default();
    use iced_x86::Formatter;
    let mut pos = 0usize;
    while decoder.can_decode() {
        decoder.decode_out(&mut instr);
        output.clear();
        formatter.format(&instr, &mut output);
        let addr = instr.ip();
        let len = instr.len() as usize;
        let bytes_hex: String = code[pos..pos + len]
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        pos += len;
        if bitness == 64 {
            lines.push(format!("{:016X}  {:30}  {}", addr, bytes_hex, output));
        } else {
            lines.push(format!("{:08X}  {:30}  {}", addr, bytes_hex, output));
        }
    }
    lines
}
