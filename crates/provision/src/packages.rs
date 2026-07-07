use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::{Package, PackageType};

/// 解析 packages.conf 文件
pub fn parse_packages_conf(path: &PathBuf) -> Result<Vec<Package>> {
    let content = std::fs::read_to_string(path).context("Failed to read packages.conf")?;

    let mut packages = Vec::new();
    let mut current_group = String::new();

    for line in content.lines() {
        let line = line.trim();

        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 解析组名
        if line.starts_with('[') && line.ends_with(']') {
            current_group = line[1..line.len() - 1].to_string();
            continue;
        }

        // 解析包定义
        if let Some(package) = parse_package_line(line, &current_group)? {
            packages.push(package);
        }
    }

    Ok(packages)
}

/// 解析单行包定义
fn parse_package_line(line: &str, group: &str) -> Result<Option<Package>> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid package line: {}", line);
    }

    let pkg_type = match parts[0].trim() {
        "pkg" => PackageType::Pkg,
        "bin" => PackageType::Bin,
        _ => anyhow::bail!("Unknown package type: {}", parts[0]),
    };

    let name = parts[1].trim().to_string();
    let desc_rest = parts[2].trim();

    // 解析描述和跨发行版包名覆盖
    let mut description = desc_rest.to_string();
    let mut apt_name = None;
    let mut yum_name = None;
    let mut apk_name = None;

    if desc_rest.contains(':') {
        let desc_parts: Vec<&str> = desc_rest.splitn(4, ':').collect();
        description = desc_parts[0].trim().to_string();

        if desc_parts.len() > 1 && !desc_parts[1].trim().is_empty() {
            apt_name = Some(desc_parts[1].trim().to_string());
        }
        if desc_parts.len() > 2 && !desc_parts[2].trim().is_empty() {
            yum_name = Some(desc_parts[2].trim().to_string());
        }
        if desc_parts.len() > 3 && !desc_parts[3].trim().is_empty() {
            apk_name = Some(desc_parts[3].trim().to_string());
        }
    }

    Ok(Some(Package {
        name,
        pkg_type,
        description,
        group: group.to_string(),
        apt_name,
        yum_name,
        apk_name,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_packages_conf() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Comment").unwrap();
        writeln!(temp_file, "[essential]").unwrap();
        writeln!(temp_file, "pkg|vim|文本编辑器").unwrap();
        writeln!(temp_file, "pkg|git|版本控制:git|git|git").unwrap();
        writeln!(temp_file, "[tools]").unwrap();
        writeln!(temp_file, "bin|tig|Git 浏览器").unwrap();

        let packages = parse_packages_conf(&temp_file.path().to_path_buf()).unwrap();

        assert_eq!(packages.len(), 3);
        assert_eq!(packages[0].name, "vim");
        assert_eq!(packages[0].pkg_type, PackageType::Pkg);
        assert_eq!(packages[0].group, "essential");
        assert_eq!(packages[1].name, "git");
        assert_eq!(packages[1].apt_name, Some("git".to_string()));
        assert_eq!(packages[2].name, "tig");
        assert_eq!(packages[2].pkg_type, PackageType::Bin);
    }
}
