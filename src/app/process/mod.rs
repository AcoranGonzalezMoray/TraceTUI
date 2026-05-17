use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::System;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub command_line: Option<String>,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub start_time: Option<DateTime<Utc>>,
    pub status: String,
}
#[derive(Debug)]
pub struct ProcessManager {
    system: System,
    processes: Vec<ProcessInfo>,
}
impl ProcessManager {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            processes: Vec::new(),
        }
    }
    pub fn refresh_processes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.system.refresh_all();
        self.processes.clear();
        for (pid, process) in self.system.processes() {
            let process_info = ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                path: process.exe().map(|p| p.to_string_lossy().to_string()),
                command_line: Some(process.cmd().join(" ")),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                start_time: Some(DateTime::from(
                    std::time::SystemTime::UNIX_EPOCH
                        + std::time::Duration::from_secs(process.start_time()),
                )),
                status: format!("{:?}", process.status()),
            };
            self.processes.push(process_info);
        }
        Ok(())
    }
    pub fn get_all_processes(&self) -> &[ProcessInfo] {
        &self.processes
    }
    pub fn kill_process(&mut self, pid: u32) -> Result<bool, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            if !self
                .system
                .processes()
                .contains_key(&sysinfo::Pid::from_u32(pid))
            {
                self.system.refresh_processes();
                if !self
                    .system
                    .processes()
                    .contains_key(&sysinfo::Pid::from_u32(pid))
                {
                    return Err("Process not found or already terminated".into());
                }
            }

            let output = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output()?;

            let stdout_msg = String::from_utf8_lossy(&output.stdout);
            let stderr_msg = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                Ok(true)
            } else {
                let combined = format!("{}{}", stdout_msg, stderr_msg);
                if combined.contains("Access is denied") || combined.contains("Access denied") {
                    Err("Access denied. Run TraceTUI as Administrator".into())
                } else if combined.contains("not found") || combined.contains("No tasks running") {
                    Err("Process not found or already terminated".into())
                } else if combined.contains("The process")
                    && combined.contains("could not be terminated")
                {
                    Err("Process is protected and cannot be terminated".into())
                } else {
                    Err(format!("Failed: {}", combined.trim()).into())
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output()?;

            if output.status.success() {
                Ok(true)
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);

                if error_msg.contains("Operation not permitted") {
                    Err("Permission denied. Try running with sudo".into())
                } else if error_msg.contains("No such process") {
                    Err("Process not found or already terminated".into())
                } else {
                    Err(format!("Failed to kill process: {}", error_msg).into())
                }
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err("Unsupported platform".into())
        }
    }

    pub fn kill_connections(&self, pid: u32) -> Result<usize, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            let check_script = format!(
                "$ErrorActionPreference = 'Stop'; \
                 try {{ \
                     $connections = Get-NetTCPConnection -OwningProcess {} -State Established -ErrorAction Stop; \
                     Write-Output $connections.Count \
                 }} catch {{ \
                     Write-Output '0' \
                 }}",
                pid
            );

            let check_output = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", &check_script])
                .output()?;

            let conn_count_str = String::from_utf8_lossy(&check_output.stdout)
                .trim()
                .to_string();
            let total_connections = conn_count_str.parse::<usize>().unwrap_or(0);

            if total_connections == 0 {
                return Ok(0);
            }

            let kill_script = format!(
                "$ErrorActionPreference = 'SilentlyContinue'; \
                 $connections = Get-NetTCPConnection -OwningProcess {} -State Established; \
                 $count = 0; \
                 foreach ($conn in $connections) {{ \
                     try {{ \
                         $conn | Remove-NetTCPConnection -Confirm:$false -ErrorAction Stop; \
                         $count++; \
                     }} catch {{ }} \
                 }}; \
                 Write-Output $count",
                pid
            );

            let output = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", &kill_script])
                .output()?;

            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                return Err(format!("PowerShell failed: {}", error_msg).into());
            }

            let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let closed_count = output_str.parse::<usize>().unwrap_or(0);

            if closed_count == 0 && total_connections > 0 {
                return Err(format!(
                    "Windows is protecting {} connection(s) for this process.\n\
                     This is normal for UWP apps, browsers, and system services.\n\
                     To force close: Kill the process instead (X key)",
                    total_connections
                )
                .into());
            }

            Ok(closed_count)
        }
        #[cfg(target_os = "linux")]
        {
            let ss_output = Command::new("ss")
                .args(["-Ktnp", &format!("( sport = :0 ) and ( pid = {} )", pid)])
                .output();

            match ss_output {
                Ok(output) if output.status.success() => {
                    let count_output = Command::new("ss").args(["-tnp"]).output()?;

                    let output_str = String::from_utf8_lossy(&count_output.stdout);
                    let mut count = 0;

                    for line in output_str.lines() {
                        if line.contains(&format!("pid={}", pid)) {
                            count += 1;
                        }
                    }

                    Ok(count)
                }
                _ => {
                    let ss_output = Command::new("ss")
                        .args(["-tnp"])
                        .output()
                        .or_else(|_| Command::new("netstat").args(["-tnp"]).output())?;

                    if !ss_output.status.success() {
                        return Err("Failed to get network connections".into());
                    }

                    let output_str = String::from_utf8_lossy(&ss_output.stdout);
                    let mut closed_count = 0;

                    for line in output_str.lines() {
                        if line.contains(&format!("pid={}", pid))
                            || line.contains(&format!("{}/", pid))
                        {
                            closed_count += 1;
                        }
                    }
                    if closed_count > 0 {
                        Ok(closed_count)
                    } else {
                        Err("No connections found or insufficient permissions".into())
                    }
                }
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err("Unsupported platform".into())
        }
    }
}
