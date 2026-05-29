use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use std::{fs, io, process::Command};

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub device: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

impl DiskInfo {
    pub fn usage_pct(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            self.used_bytes as f64 / self.total_bytes as f64 * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub extension: String,
}

pub const FILE_EXTENSION_FILTERS: &[(&str, &[&str])] = &[
    ("\u{f15b}", &[]),
    (
        "\u{f1c5}",
        &[
            "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "tiff", "tif",
        ],
    ),
    (
        "\u{f15c}",
        &[
            "txt", "md", "pdf", "doc", "docx", "csv", "json", "toml", "yml", "yaml", "xml", "html",
            "htm", "css", "sql", "log",
        ],
    ),
    (
        "\u{f1c9}",
        &[
            "rs", "py", "js", "ts", "go", "c", "cpp", "h", "hpp", "java", "rb", "php", "swift",
            "kt", "r", "pl", "lua",
        ],
    ),
    ("\u{f1c6}", &["zip", "tar", "gz", "bz2", "xz", "7z", "rar"]),
    (
        "\u{f1c7}",
        &["mp3", "wav", "flac", "ogg", "aac", "wma", "m4a"],
    ),
    (
        "\u{f1c8}",
        &["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm"],
    ),
];

pub fn extension_filter_label(idx: usize) -> &'static str {
    match idx {
        0 => "storage.extension_all",
        1 => "storage.extension_images",
        2 => "storage.extension_documents",
        3 => "storage.extension_code",
        4 => "storage.extension_archives",
        5 => "storage.extension_audio",
        6 => "storage.extension_video",
        _ => "storage.extension_all",
    }
}

pub struct StorageManager;

impl StorageManager {
    pub fn list_disks() -> Vec<DiskInfo> {
        let mut disks = Vec::new();
        #[cfg(windows)]
        {
            for letter in 'A'..='Z' {
                let path = format!("{}:\\", letter);
                if Path::new(&path).exists() {
                    if let Some(info) = Self::get_disk_info(&path) {
                        disks.push(info);
                    }
                }
            }
        }
        #[cfg(unix)]
        {
            if let Ok(content) = fs::read_to_string("/proc/mounts") {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let _device = parts[0];
                        let mount = parts[1];
                        let fstype = parts[2];
                        if !fstype.starts_with("proc")
                            && !fstype.starts_with("sys")
                            && !fstype.starts_with("dev")
                            && !fstype.starts_with("tmp")
                            && !fstype.starts_with("cgroup")
                            && !fstype.starts_with("sunrpc")
                            && !fstype.starts_with("rpc")
                            && mount != "/dev"
                            && mount != "/dev/shm"
                            && mount != "/sys"
                            && mount != "/proc"
                        {
                            if let Some(info) = Self::get_disk_info(mount) {
                                disks.push(info);
                            }
                        }
                    }
                }
            }
        }
        disks.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
        disks
    }

    fn get_disk_info(path: &str) -> Option<DiskInfo> {
        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::iter;
            use std::os::windows::ffi::OsStrExt;
            #[allow(clippy::upper_case_acronyms)]
            type BOOL = i32;
            #[allow(non_camel_case_types)]
            type ULARGE_INTEGER = u64;

            extern "system" {
                fn GetDiskFreeSpaceExW(
                    lpDirectoryName: *const u16,
                    lpFreeBytesAvailable: *mut ULARGE_INTEGER,
                    lpTotalNumberOfBytes: *mut ULARGE_INTEGER,
                    lpTotalNumberOfFreeBytes: *mut ULARGE_INTEGER,
                ) -> BOOL;
                fn GetVolumeInformationW(
                    lpRootPathName: *const u16,
                    lpVolumeNameBuffer: *mut u16,
                    nVolumeNameSize: u32,
                    lpVolumeSerialNumber: *mut u32,
                    lpMaximumComponentLength: *mut u32,
                    lpFileSystemFlags: *mut u32,
                    lpFileSystemNameBuffer: *mut u16,
                    nFileSystemNameSize: u32,
                ) -> BOOL;
            }

            let wide: Vec<u16> = OsStr::new(path)
                .encode_wide()
                .chain(iter::once(0))
                .collect();

            let mut free_avail: u64 = 0;
            let mut total: u64 = 0;
            let mut total_free: u64 = 0;

            let result = unsafe {
                GetDiskFreeSpaceExW(wide.as_ptr(), &mut free_avail, &mut total, &mut total_free)
            };

            if result == 0 {
                return None;
            }

            let mut fs_name_buf = [0u16; 32];
            let mut _vol_name = [0u16; 32];
            let mut _serial = 0u32;
            let mut _max_comp = 0u32;
            let mut _flags = 0u32;

            unsafe {
                GetVolumeInformationW(
                    wide.as_ptr(),
                    &mut _vol_name[0],
                    32,
                    &mut _serial,
                    &mut _max_comp,
                    &mut _flags,
                    &mut fs_name_buf[0],
                    32,
                );
            }

            let fs_name = String::from_utf16_lossy(&fs_name_buf)
                .trim_end_matches(char::from(0))
                .to_string();

            let device = path.trim_end_matches('\\').to_string();
            let used = total.saturating_sub(total_free);

            Some(DiskInfo {
                mount_point: path.to_string(),
                device,
                fs_type: if fs_name.is_empty() {
                    "NTFS".to_string()
                } else {
                    fs_name
                },
                total_bytes: total,
                used_bytes: used,
                free_bytes: total_free,
            })
        }
        #[cfg(unix)]
        {
            use std::ffi::CString;
            use std::mem;

            #[repr(C)]
            struct statvfs {
                f_bsize: u64,
                f_frsize: u64,
                f_blocks: u64,
                f_bfree: u64,
                f_bavail: u64,
                f_files: u64,
                f_ffree: u64,
                f_favail: u64,
                f_fsid: u64,
                f_flag: u64,
                f_namemax: u64,
            }

            extern "C" {
                fn statvfs(path: *const i8, buf: *mut statvfs) -> i32;
            }

            let cpath = CString::new(path).ok()?;
            let mut buf: statvfs = unsafe { mem::zeroed() };

            let result = unsafe { statvfs(cpath.as_ptr(), &mut buf) };
            if result == 0 {
                let total = buf.f_blocks * buf.f_frsize;
                let free = buf.f_bfree * buf.f_frsize;
                let used = total.saturating_sub(free);
                Some(DiskInfo {
                    mount_point: path.to_string(),
                    device: String::new(),
                    fs_type: String::new(),
                    total_bytes: total,
                    used_bytes: used,
                    free_bytes: free,
                })
            } else {
                None
            }
        }
    }

    pub fn list_directory(path: &Path) -> io::Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let ft = entry.file_type()?;
                let metadata = entry.metadata()?;
                let name = entry.file_name().to_string_lossy().to_string();
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|t| {
                        let secs = t.duration_since(UNIX_EPOCH).ok()?.as_secs();
                        let s = secs as i64;
                        let (y, mo, d, h, mi) = Self::from_unix(s);
                        Some(format!("{:04}-{:02}-{:02} {:02}:{:02}", y, mo, d, h, mi))
                    })
                    .unwrap_or_default();

                let ext = if ft.is_dir() {
                    String::new()
                } else {
                    entry
                        .path()
                        .extension()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase()
                };

                let size = metadata.len();

                entries.push(FileEntry {
                    name,
                    path: entry.path(),
                    is_dir: ft.is_dir(),
                    size,
                    modified,
                    extension: ext,
                });
            }
        }

        entries.sort_unstable_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });
        Ok(entries)
    }

    fn from_unix(secs: i64) -> (i64, i64, i64, i64, i64) {
        let mut s = secs;
        let y400 = 400 * 365 + 97;
        let y100 = 100 * 365 + 24;
        let y4 = 4 * 365 + 1;
        let y1 = 365;

        s += 11676096000;
        let mut n400 = s / (y400 * 86400);
        s %= y400 * 86400;
        if s < 0 {
            s += y400 * 86400;
            n400 -= 1;
        }

        let mut n100 = s / (y100 * 86400);
        s %= y100 * 86400;
        if n100 == 4 {
            n100 = 3;
            s = y100 * 86400 - 1;
        }

        let n4 = s / (y4 * 86400);
        s %= y4 * 86400;

        let mut n1 = s / (y1 * 86400);
        s %= y1 * 86400;
        if n1 == 4 {
            n1 = 3;
            s = y1 * 86400 - 1;
        }

        let year = 1600 + n400 * 400 + n100 * 100 + n4 * 4 + n1;
        let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);

        let mut month = 0i64;
        let mut remaining = s / 86400;
        for (i, &md) in month_days.iter().enumerate() {
            let days = md + if i == 1 && is_leap { 1 } else { 0 };
            if remaining < days {
                month = (i + 1) as i64;
                break;
            }
            remaining -= days;
        }

        let day = remaining + 1;
        let hour = (s % 86400) / 3600;
        let min = (s % 3600) / 60;

        (year, month, day, hour, min)
    }

    pub fn read_file(path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    pub fn is_text_file(ext: &str) -> bool {
        matches!(
            ext,
            "txt"
                | "rs"
                | "json"
                | "md"
                | "log"
                | "toml"
                | "yml"
                | "yaml"
                | "xml"
                | "html"
                | "css"
                | "js"
                | "ts"
                | "py"
                | "rb"
                | "sh"
                | "bat"
                | "ps1"
                | "cfg"
                | "ini"
                | "conf"
                | "env"
                | "csv"
                | "sql"
                | "lua"
                | "go"
                | "c"
                | "h"
                | "cpp"
                | "hpp"
                | "java"
                | "kt"
                | "swift"
                | "r"
                | "pl"
                | "php"
                | "vue"
                | "svelte"
                | "tsx"
                | "jsx"
                | "dockerfile"
                | "makefile"
                | "gradle"
                | "tf"
                | "sln"
                | "csproj"
                | "lock"
                | "gitignore"
                | "editorconfig"
                | "prettierrc"
                | "eslintrc"
        )
    }

    pub fn sort_entries(entries: &mut [FileEntry], sort_mode: crate::app::types::FileSortMode) {
        entries.sort_unstable_by(|a, b| match sort_mode {
            crate::app::types::FileSortMode::ByName => {
                if a.is_dir != b.is_dir {
                    b.is_dir.cmp(&a.is_dir)
                } else {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            }
            crate::app::types::FileSortMode::BySize => {
                if a.is_dir != b.is_dir {
                    b.is_dir.cmp(&a.is_dir)
                } else {
                    b.size.cmp(&a.size)
                }
            }
            crate::app::types::FileSortMode::ByDate => {
                if a.is_dir != b.is_dir {
                    b.is_dir.cmp(&a.is_dir)
                } else {
                    b.modified.cmp(&a.modified)
                }
            }
        });
    }

    pub fn is_image_file(ext: &str) -> bool {
        matches!(
            ext,
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif"
        )
    }
}

pub fn render_image_preview(path: &Path) -> Option<Vec<String>> {
    #[cfg(windows)]
    {
        let script = format!(
            r#"
Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Image]::FromFile('{}')
$bmp = New-Object System.Drawing.Bitmap($img)
$w = [math]::Min($bmp.Width, 60)
$h = [math]::Min($bmp.Height, 40)
$thumb = New-Object System.Drawing.Bitmap($w, $h)
$g = [System.Drawing.Graphics]::FromImage($thumb)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.DrawImage($bmp, 0, 0, $w, $h)
$blockChar = [char]0x2580
for ($y = 0; $y -lt $thumb.Height; $y += 2) {{
    $line = ""
    for ($x = 0; $x -lt $thumb.Width; $x++) {{
        $tp = $thumb.GetPixel($x, $y)
        $bp = if ($y + 1 -lt $thumb.Height) {{ $thumb.GetPixel($x, $y + 1) }} else {{ [System.Drawing.Color]::Transparent }}
        if ($tp.A -eq 0 -and $bp.A -eq 0) {{ $line += "  " }}
        else {{
            $fg = if ($tp.A -gt 0) {{ "$([char]27)[38;2;$($tp.R);$($tp.G);$($tp.B)m" }} else {{ "$([char]27)[39m" }}
            $bg = if ($bp.A -gt 0) {{ "$([char]27)[48;2;$($bp.R);$($bp.G);$($bp.B)m" }} else {{ "$([char]27)[49m" }}
            $line += "${{fg}}${{bg}}$blockChar"
        }}
    }}
    Write-Host "$line$([char]27)[0m"
}}
$thumb.Dispose(); $g.Dispose(); $bmp.Dispose(); $img.Dispose()
"#,
            path.display().to_string().replace('\'', "''")
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
            if !lines.is_empty() {
                return Some(lines);
            }
        }
        None
    }
    #[cfg(unix)]
    {
        if let Ok(out) = Command::new("chafa")
            .args([
                "--symbols",
                "block",
                "-c",
                "240",
                "-s",
                "60x40",
                "--format",
                "symbols",
            ])
            .arg(path.as_os_str())
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
                if !lines.is_empty() {
                    return Some(lines);
                }
            }
        }

        if let Ok(out) = Command::new("catimg")
            .args(["-w", "60", "-r", "2"])
            .arg(path.as_os_str())
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
                if !lines.is_empty() {
                    return Some(lines);
                }
            }
        }

        if let Ok(out) = Command::new("python3")
        .args(["-c", &format!(
            r#"import sys; sys.path.insert(0,''); from PIL import Image; i=Image.open('{}'); i=i.resize((60,int(i.height*60/i.width))); px=i.load(); w,h=i.size; b=chr(0x2580)
    for y in range(0,h,2):
     l=''
     for x in range(w):
      tp=px[x,y]; bp=px[x,min(y+1,h-1)]
      l+=f'\x1b[38;2;{{tp[0]}};{{tp[1]}};{{tp[2]}}m\x1b[48;2;{{bp[0]}};{{bp[1]}};{{bp[2]}}m{{b}}\x1b[0m'
     print(l)"#,
            path.display().to_string().replace('\'', "'\\''")
        )])
        .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
                if !lines.is_empty() { return Some(lines); }
            }
        }
        None
    }
    #[cfg(not(any(windows, unix)))]
    {
        None
    }
}

pub fn fmt_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit])
    }
}
