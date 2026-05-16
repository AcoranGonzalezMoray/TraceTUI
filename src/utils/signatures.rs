use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
static SIGNATURE_CACHE: Lazy<Mutex<HashMap<String, SignatureStatus>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SignatureStatus {
    Valid,
    Invalid,
    Unsigned,
    Unknown,
}
pub struct SignatureVerifier;
impl SignatureVerifier {
    pub fn verify(path: &str) -> SignatureStatus {
        if path.is_empty() || path == "Unknown" {
            return SignatureStatus::Unknown;
        }
        {
            let cache = SIGNATURE_CACHE.lock().unwrap();
            if let Some(status) = cache.get(path) {
                return status.clone();
            }
        }
        let status = {
            #[cfg(windows)]
            {
                Self::check_signature_windows(path)
            }
            #[cfg(target_os = "linux")]
            {
                Self::check_signature_linux(path)
            }
            #[cfg(not(any(windows, target_os = "linux")))]
            {
                SignatureStatus::Unknown
            }
        };
        let mut cache = SIGNATURE_CACHE.lock().unwrap();
        cache.insert(path.to_string(), status.clone());
        status
    }
    #[cfg(windows)]
    fn check_signature_windows(path: &str) -> SignatureStatus {
        let script = format!("(Get-AuthenticodeSignature '{}').Status", path);
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                match stdout.as_str() {
                    "Valid" => SignatureStatus::Valid,
                    "NotSigned" => SignatureStatus::Unsigned,
                    "HashMismatch" | "NotTrusted" => SignatureStatus::Invalid,
                    _ => SignatureStatus::Unsigned,
                }
            }
            Err(_) => SignatureStatus::Unknown,
        }
    }
    #[cfg(target_os = "linux")]
    fn check_signature_linux(path: &str) -> SignatureStatus {
        if !std::path::Path::new(path).exists() {
            return SignatureStatus::Unknown;
        }
        let metadata = std::fs::metadata(path);
        if let Ok(meta) = metadata {
            use std::os::unix::fs::PermissionsExt;
            let permissions = meta.permissions();
            if permissions.mode() & 0o111 != 0 {
                let output = Command::new("file").args([path]).output();
                if let Ok(out) = output {
                    let info = String::from_utf8_lossy(&out.stdout).to_lowercase();
                    if info.contains("elf") {
                        return SignatureStatus::Unsigned;
                    }
                }
            }
        }
        SignatureStatus::Unknown
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    fn check_signature_linux(_path: &str) -> SignatureStatus {
        SignatureStatus::Unknown
    }
}
