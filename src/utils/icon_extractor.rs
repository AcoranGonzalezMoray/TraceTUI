use crate::config;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Command;
#[derive(Debug)]
pub struct IconCache {
    cache: LruCache<String, IconData>,
}
#[derive(Debug, Clone)]
pub enum IconData {
    Text(String),
}
impl IconCache {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(config::LRU_CACHE_SIZE).unwrap()),
        }
    }
    pub fn get_icon(&mut self, exe_path: &str, process_name: &str) -> String {
        if let Some(icon) = self.cache.get(exe_path) {
            return match icon {
                IconData::Text(text) => text.clone(),
            };
        }
        let icon_data = {
            #[cfg(windows)]
            {
                self.extract_icon_windows(exe_path)
            }
            #[cfg(target_os = "linux")]
            {
                self.extract_icon_linux(exe_path, process_name)
            }
            #[cfg(not(any(windows, target_os = "linux")))]
            {
                None
            }
        };
        if let Some(icon_data) = icon_data {
            self.cache.put(exe_path.to_string(), icon_data.clone());
            match icon_data {
                IconData::Text(text) => text,
            }
        } else if cfg!(target_os = "linux") {
            String::new()
        } else {
            generate_fallback_icon(process_name)
        }
    }
    pub fn insert_icon(&mut self, exe_path: &str, icon: String) {
        self.cache.put(exe_path.to_string(), IconData::Text(icon));
    }
    #[cfg(windows)]
    fn script_path() -> String {
        let relative = config::ICON_EXTRACTOR_SCRIPT;
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let p = dir.join(relative);
                if p.exists() {
                    return p.to_string_lossy().to_string();
                }
            }
        }
        relative.to_string()
    }
    #[cfg(windows)]
    fn extract_icon_windows(&self, exe_path: &str) -> Option<IconData> {
        if !Path::new(exe_path).exists() {
            return None;
        }
        let script_path = Self::script_path();
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script_path,
                "-ExePath",
                exe_path,
                "-Width",
                config::ICON_EXTRACTOR_WIDTH,
            ])
            .output();
        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !stdout.is_empty() {
                    Some(IconData::Text(stdout))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    #[cfg(target_os = "linux")]
    fn extract_icon_linux(&self, _exe_path: &str, process_name: &str) -> Option<IconData> {
        let name_lower = process_name.to_lowercase();
        let icon_name = get_desktop_icon_name(&name_lower);
        if let Some(ref icon_name) = icon_name {
            if let Some(path) = find_icon_path_linux(icon_name, 24) {
                if let Some(ansi) = convert_icon_to_ansi(&path, 24) {
                    return Some(IconData::Text(ansi));
                }
            }
        }
        None
    }
}
#[cfg(target_os = "linux")]
fn get_desktop_icon_name(name: &str) -> Option<String> {
    let candidates = vec![
        format!("/usr/share/applications/{}.desktop", name),
        format!("/usr/share/applications/{}.desktop", name.replace('.', "-")),
        format!("/usr/share/applications/{}.desktop", name.replace('.', "")),
    ];
    for path in candidates {
        if Path::new(&path).exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                for line in contents.lines() {
                    if line.starts_with("Icon=") {
                        return Some(line.trim_start_matches("Icon=").trim().to_string());
                    }
                }
            }
        }
    }
    let appdir = Path::new("/usr/share/applications");
    if let Ok(entries) = std::fs::read_dir(appdir) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.ends_with(".desktop") && fname.contains(&name.replace('.', "-")) {
                if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                    for line in contents.lines() {
                        if line.starts_with("Icon=") {
                            return Some(line.trim_start_matches("Icon=").trim().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
#[cfg(target_os = "linux")]
fn find_icon_path_linux(icon_name: &str, desired_size: u32) -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let icon_dirs = vec![
        format!("{}/.local/share/icons", home),
        "/usr/share/icons".to_string(),
        "/usr/local/share/icons".to_string(),
    ];
    let sizes: [u32; 7] = [256, 128, 64, 48, 32, 24, 16];
    for dir in &icon_dirs {
        for &size in &sizes {
            let p = format!("{}/hicolor/{}x{}/apps/{}.png", dir, size, size, icon_name);
            if Path::new(&p).exists() && size >= desired_size {
                return Some(p);
            }
        }
    }
    for dir in &icon_dirs {
        let p = format!("{}/hicolor/scalable/apps/{}.svg", dir, icon_name);
        if Path::new(&p).exists() {
            return Some(p);
        }
    }
    let mut best: Option<(String, u32)> = None;
    for dir in &icon_dirs {
        for &size in &sizes {
            let p = format!("{}/hicolor/{}x{}/apps/{}.png", dir, size, size, icon_name);
            if Path::new(&p).exists() {
                match &best {
                    Some((_, bs)) if size > *bs => best = Some((p, size)),
                    None => best = Some((p, size)),
                    _ => {}
                }
            }
        }
    }
    if let Some((path, _)) = best {
        return Some(path);
    }
    let pixmaps = ["/usr/share/pixmaps", "/usr/local/share/pixmaps"];
    for dir in &pixmaps {
        for ext in &["png", "svg", "xpm"] {
            let p = format!("{}/{}.{}", dir, icon_name, ext);
            if Path::new(&p).exists() {
                return Some(p);
            }
        }
    }
    let other_themes = [
        "gnome",
        "Adwaita",
        "Papirus",
        "breeze",
        "Mint-X",
        "elementary",
    ];
    for dir in &icon_dirs {
        for theme in &other_themes {
            for &size in &sizes {
                let p = format!("{}/{}/{}x{}/apps/{}.png", dir, theme, size, size, icon_name);
                if Path::new(&p).exists() && size >= desired_size {
                    return Some(p);
                }
                let p = format!("{}/{}/{}x{}/apps/{}.svg", dir, theme, size, size, icon_name);
                if Path::new(&p).exists() && size >= desired_size {
                    return Some(p);
                }
            }
            let p = format!("{}/{}/scalable/apps/{}.svg", dir, theme, icon_name);
            if Path::new(&p).exists() {
                return Some(p);
            }
        }
    }
    None
}
#[cfg(target_os = "linux")]
fn try_convert_svg(svg_path: &str, width: u32) -> Option<String> {
    let tmp_dir = std::env::temp_dir();
    let output_png = tmp_dir.join(format!("tracetui_icon_{}.png", std::process::id()));
    let status = Command::new("rsvg-convert")
        .args([
            "-w",
            &width.to_string(),
            svg_path,
            "-o",
            &output_png.to_string_lossy(),
        ])
        .status();
    if let Ok(status) = status {
        if status.success() && output_png.exists() {
            let result = convert_png_to_ansi(&output_png.to_string_lossy(), width);
            let _ = std::fs::remove_file(&output_png);
            return result;
        }
    }
    let status = Command::new("convert")
        .args([
            svg_path,
            "-resize",
            &format!("{}x{}", width, width),
            &output_png.to_string_lossy(),
        ])
        .status();
    if let Ok(status) = status {
        if status.success() && output_png.exists() {
            let result = convert_png_to_ansi(&output_png.to_string_lossy(), width);
            let _ = std::fs::remove_file(&output_png);
            return result;
        }
    }
    let _ = std::fs::remove_file(&output_png);
    None
}
#[cfg(target_os = "linux")]
fn convert_png_to_ansi(png_path: &str, width: u32) -> Option<String> {
    use std::io::Write;
    let tmp_dir = std::env::temp_dir();
    let script_path = tmp_dir.join(format!("tracetui_png2ansi_{}.py", std::process::id()));
    let mut file = std::fs::File::create(&script_path).ok()?;
    file.write_all(PYTHON_PNG_TO_ANSI.as_bytes()).ok()?;
    drop(file);
    let output = Command::new("python3")
        .args([&script_path.to_string_lossy(), png_path, &width.to_string()])
        .output();
    let _ = std::fs::remove_file(&script_path);
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stdout.is_empty() {
                return Some(stdout);
            }
        }
    }
    None
}
#[cfg(target_os = "linux")]
fn convert_icon_to_ansi(icon_path: &str, width: u32) -> Option<String> {
    let lower = icon_path.to_lowercase();
    if lower.ends_with(".svg") {
        return try_convert_svg(icon_path, width);
    }
    if lower.ends_with(".png") {
        return convert_png_to_ansi(icon_path, width);
    }
    convert_png_to_ansi(icon_path, width)
}
#[cfg(target_os = "linux")]
const PYTHON_PNG_TO_ANSI: &str = r##"import struct, zlib, sys
def main():
    if len(sys.argv) < 3:
        sys.exit(1)
    filepath = sys.argv[1]
    out_w = int(sys.argv[2])
    if out_w < 1:
        out_w = 24
    try:
        with open(filepath, 'rb') as f:
            data = f.read()
    except:
        sys.exit(1)
    if len(data) < 8 or data[:8] != b'\x89PNG\r\n\x1a\n':
        sys.exit(1)
    pos = 8
    img_w = img_h = 0
    bit_depth = color_type = 0
    idat_data = b''
    palette = []
    trns_data = b''
    while pos + 8 <= len(data):
        length = struct.unpack('>I', data[pos:pos+4])[0]
        chunk_type = data[pos+4:pos+8]
        chunk_data = data[pos+8:pos+8+length]
        if chunk_type == b'IHDR':
            if len(chunk_data) >= 13:
                img_w = struct.unpack('>I', chunk_data[0:4])[0]
                img_h = struct.unpack('>I', chunk_data[4:8])[0]
                bit_depth = chunk_data[8]
                color_type = chunk_data[9]
                if chunk_data[12] != 0:
                    sys.exit(1)
        elif chunk_type == b'PLTE':
            palette = [chunk_data[i:i+3] for i in range(0, len(chunk_data), 3)]
        elif chunk_type == b'tRNS':
            trns_data = chunk_data
        elif chunk_type == b'IDAT':
            idat_data += chunk_data
        elif chunk_type == b'IEND':
            break
        pos += 12 + length
    if img_w == 0 or img_h == 0 or not idat_data:
        sys.exit(1)
    if bit_depth != 8:
        sys.exit(1)
    try:
        raw = zlib.decompress(idat_data)
    except:
        sys.exit(1)
    if color_type == 2:
        bpp = 3
    elif color_type == 6:
        bpp = 4
    elif color_type == 3:
        bpp = 1
    elif color_type == 0:
        bpp = 1
    elif color_type == 4:
        bpp = 2
    else:
        sys.exit(1)
    stride = 1 + img_w * bpp
    if len(raw) < stride * img_h:
        sys.exit(1)
    rgba = []
    prev_row = [0] * (img_w * bpp)
    for row_idx in range(img_h):
        offset = row_idx * stride
        filter_type = raw[offset]
        row_data = bytearray(raw[offset+1:offset+stride])
        for i in range(len(row_data)):
            val = row_data[i]
            left = row_data[i - bpp] if i >= bpp else 0
            up = prev_row[i]
            up_left = prev_row[i - bpp] if i >= bpp else 0
            if filter_type == 0:
                recon = val
            elif filter_type == 1:
                recon = (val + left) & 0xFF
            elif filter_type == 2:
                recon = (val + up) & 0xFF
            elif filter_type == 3:
                recon = (val + (left + up) // 2) & 0xFF
            elif filter_type == 4:
                p = left + up - up_left
                pa = abs(p - left)
                pb = abs(p - up)
                pc = abs(p - up_left)
                if pa <= pb and pa <= pc:
                    pr = left
                elif pb <= pc:
                    pr = up
                else:
                    pr = up_left
                recon = (val + pr) & 0xFF
            else:
                recon = val
            row_data[i] = recon
        for i in range(img_w):
            off = i * bpp
            if color_type == 2:
                r, g, b = row_data[off:off+3]
                a = 255
            elif color_type == 6:
                r, g, b, a = row_data[off:off+4]
            elif color_type == 3:
                idx = row_data[off]
                if idx < len(palette):
                    r, g, b = palette[idx]
                    a = trns_data[idx] if idx < len(trns_data) else 255
                else:
                    r, g, b, a = 0, 0, 0, 0
            elif color_type == 0:
                r = g = b = row_data[off]
                a = 255
            elif color_type == 4:
                r = g = b = row_data[off]
                a = row_data[off+1]
            rgba.append((r, g, b, a))
        prev_row = row_data
    scale_w = out_w
    scale_h = max(2, img_h * out_w // img_w)
    if scale_h % 2 != 0:
        scale_h += 1
    ESC = '\x1b'
    for cy in range(scale_h // 2):
        line = ''
        for cx in range(scale_w):
            sx = min(cx * img_w // scale_w, img_w - 1)
            sy_top = min((cy * 2) * img_h // scale_h, img_h - 1)
            sy_bot = min((cy * 2 + 1) * img_h // scale_h, img_h - 1)
            top_idx = sy_top * img_w + sx
            bot_idx = sy_bot * img_w + sx
            if top_idx >= len(rgba) or bot_idx >= len(rgba):
                line += ' '
                continue
            tr, tg, tb, ta = rgba[top_idx]
            br, bg, bb, ba = rgba[bot_idx]
            if ta == 0 and ba == 0:
                line += ' '
            else:
                fg = f'{ESC}[39m' if ta == 0 else f'{ESC}[38;2;{tr};{tg};{tb}m'
                bg = f'{ESC}[49m' if ba == 0 else f'{ESC}[48;2;{br};{bg};{bb}m'
                if ta == 0:
                    line += f'{fg}{bg} '
                else:
                    line += f'{fg}{bg}\u2580'
        line += f'{ESC}[0m'
        print(line)
if __name__ == '__main__':
    main()
"##;
fn generate_fallback_icon(process_name: &str) -> String {
    process_name.chars().take(3).collect::<String>()
}
