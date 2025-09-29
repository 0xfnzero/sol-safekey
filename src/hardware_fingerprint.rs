use ring::digest;
use std::process::Command;

/// 硬件指纹结构
#[derive(Debug, Clone)]
pub struct HardwareFingerprint {
    pub fingerprint: String,
}

impl HardwareFingerprint {
    /// 收集硬件指纹
    pub fn collect() -> Result<Self, String> {
        let mut components = Vec::new();

        // 1. CPU 信息
        if let Ok(cpu_info) = Self::get_cpu_info() {
            components.push(format!("CPU:{}", cpu_info));
        }

        // 2. 主板序列号 (macOS)
        if let Ok(serial) = Self::get_system_serial() {
            components.push(format!("SERIAL:{}", serial));
        }

        // 3. MAC 地址
        if let Ok(mac) = Self::get_mac_address() {
            components.push(format!("MAC:{}", mac));
        }

        // 4. 硬盘序列号
        if let Ok(disk) = Self::get_disk_serial() {
            components.push(format!("DISK:{}", disk));
        }

        if components.is_empty() {
            return Err("无法收集硬件指纹信息".to_string());
        }

        // 组合所有硬件信息并生成指纹
        let combined = components.join("|");
        let hash = digest::digest(&digest::SHA256, combined.as_bytes());
        let fingerprint = hex::encode(hash.as_ref());

        Ok(Self { fingerprint })
    }

    /// 获取 CPU 信息
    fn get_cpu_info() -> Result<String, String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("sysctl")
                .args(&["-n", "machdep.cpu.brand_string"])
                .output()
                .map_err(|e| format!("获取CPU信息失败: {}", e))?;

            if output.status.success() {
                let info = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(info);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("model name") {
                        if let Some(name) = line.split(':').nth(1) {
                            return Ok(name.trim().to_string());
                        }
                    }
                }
            }
        }

        Err("无法获取CPU信息".to_string())
    }

    /// 获取系统序列号
    fn get_system_serial() -> Result<String, String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("ioreg")
                .args(&["-l"])
                .output()
                .map_err(|e| format!("获取系统序列号失败: {}", e))?;

            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines() {
                    if line.contains("IOPlatformSerialNumber") {
                        if let Some(serial) = line.split('=').nth(1) {
                            let serial = serial.trim().trim_matches('"').to_string();
                            if !serial.is_empty() {
                                return Ok(serial);
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/sys/class/dmi/id/product_serial") {
                let serial = content.trim().to_string();
                if !serial.is_empty() && serial != "0" {
                    return Ok(serial);
                }
            }
        }

        Err("无法获取系统序列号".to_string())
    }

    /// 获取 MAC 地址
    fn get_mac_address() -> Result<String, String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("ifconfig")
                .output()
                .map_err(|e| format!("获取MAC地址失败: {}", e))?;

            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines() {
                    if line.contains("ether") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return Ok(parts[1].to_string());
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/sys/class/net/eth0/address") {
                let mac = content.trim().to_string();
                if !mac.is_empty() {
                    return Ok(mac);
                }
            }
        }

        Err("无法获取MAC地址".to_string())
    }

    /// 获取硬盘序列号
    fn get_disk_serial() -> Result<String, String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("diskutil")
                .args(&["info", "/"])
                .output()
                .map_err(|e| format!("获取硬盘序列号失败: {}", e))?;

            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines() {
                    if line.contains("Volume UUID") || line.contains("Disk / Partition UUID") {
                        if let Some(uuid) = line.split(':').nth(1) {
                            return Ok(uuid.trim().to_string());
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("lsblk")
                .args(&["-o", "UUID", "-n"])
                .output()
                .map_err(|e| format!("获取硬盘UUID失败: {}", e))?;

            if output.status.success() {
                let uuid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !uuid.is_empty() {
                    return Ok(uuid);
                }
            }
        }

        Err("无法获取硬盘序列号".to_string())
    }

    /// 获取指纹字符串
    pub fn as_str(&self) -> &str {
        &self.fingerprint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_fingerprint() {
        match HardwareFingerprint::collect() {
            Ok(fp) => {
                println!("硬件指纹: {}", fp.as_str());
                assert!(!fp.as_str().is_empty());
                assert_eq!(fp.as_str().len(), 64); // SHA256 hex = 64 chars
            }
            Err(e) => {
                println!("警告: 无法收集硬件指纹: {}", e);
            }
        }
    }

    #[test]
    fn test_fingerprint_consistency() {
        let fp1 = HardwareFingerprint::collect();
        let fp2 = HardwareFingerprint::collect();

        match (fp1, fp2) {
            (Ok(f1), Ok(f2)) => {
                assert_eq!(f1.as_str(), f2.as_str(), "相同硬件应产生相同指纹");
            }
            _ => {
                println!("警告: 跳过一致性测试");
            }
        }
    }
}