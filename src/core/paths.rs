//! 计算项目本地数据目录位置
//!
//! 数据目录结构：
//!
//! ```text
//! <data_dir>/
//!   global_config.json     全局配置
//!   cookies/               Cookie 文件（<qq>.txt）
//!   config/                账号配置（<qq>.json）
//!   logs/                  运行日志
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use directories::ProjectDirs;

#[derive(Clone, Debug)]
pub struct Paths {
    data_dir: PathBuf,
    logs_dir: PathBuf,
    config_dir: PathBuf,
    cookies_file: PathBuf,
    global_config_file: PathBuf,
}

impl Paths {
    /// 返回 Paths 结构体
    ///
    /// # Errors
    ///
    /// 仅在无法确定系统项目目录时返回错误（例如无 HOME 环境变量）
    pub fn new() -> Result<Self> {
        let dirs = match ProjectDirs::from("io.github", "gaoyuanqi", "dld") {
            Some(d) => d,
            None => bail!("无法确定项目目录（可能 HOME 未设置）"),
        };
        Self::new_from(dirs.data_local_dir())
    }

    /// 从指定数据根目录创建 Paths 并确保子目录存在（测试用）
    pub(crate) fn new_from(data_dir: &Path) -> Result<Self> {
        let paths = Self {
            logs_dir: data_dir.join("logs"),
            config_dir: data_dir.join("config"),
            cookies_file: data_dir.join("cookies.json"),
            global_config_file: data_dir.join("global_config.json"),
            data_dir: data_dir.to_path_buf(),
        };
        paths.ensure_dirs()?;
        Ok(paths)
    }

    /// 返回项目本地cookie文件的路径
    ///
    /// | 平台       | 示例                                                                            |
    /// |-----------|---------------------------------------------------------------------------------|
    /// | Linux     | `/home/Alice/.local/share/dld/cookies.json`                                     |
    /// | Windows   | `C:\Users\Alice\AppData\Local\gaoyuanqi\dld\cookies.json`                       |
    /// | macOS     | `/Users/Alice/Library/Application Support/io.github.gaoyuanqi.dld/cookies.json` |
    pub fn cookies_file(&self) -> &Path {
        &self.cookies_file
    }

    /// 返回全局配置文件的路径
    ///
    /// | 平台       | 示例                                                                                      |
    /// |-----------|-------------------------------------------------------------------------------------------|
    /// | Linux     | `/home/Alice/.local/share/dld/global_config.json`                                         |
    /// | Windows   | `C:\Users\Alice\AppData\Local\gaoyuanqi\dld\global_config.json`                           |
    /// | macOS     | `/Users/Alice/Library/Application Support/io.github.gaoyuanqi.dld/global_config.json`     |
    pub fn global_config_file(&self) -> &Path {
        &self.global_config_file
    }

    /// 返回程序数据根目录
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// 返回项目本地日志目录的路径
    ///
    /// | 平台       | 示例                                                                    |
    /// |-----------|-------------------------------------------------------------------------|
    /// | Linux     | `/home/Alice/.local/share/dld/logs`                                     |
    /// | Windows   | `C:\Users\Alice\AppData\Local\gaoyuanqi\dld\logs`                       |
    /// | macOS     | `/Users/Alice/Library/Application Support/io.github.gaoyuanqi.dld/logs` |
    pub fn logs_dir(&self) -> &Path {
        &self.logs_dir
    }

    /// 返回账号配置目录的路径
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// 确保子目录存在
    fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.logs_dir)?;
        fs::create_dir_all(&self.config_dir)?;
        Ok(())
    }

    /// 收紧数据目录与 Cookie 文件权限（Unix 下目录 0700、Cookie 文件 0600）
    ///
    /// 幂等：每次启动调用，已收紧的路径重复设置无副作用
    /// 覆盖老用户升级前创建的宽松权限文件
    pub(crate) fn tighten_permissions(&self) -> Result<()> {
        #[cfg(unix)]
        {
            use anyhow::Context;
            use std::os::unix::fs::PermissionsExt;

            for dir in [&self.data_dir, &self.logs_dir, &self.config_dir] {
                fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
                    .with_context(|| format!("收紧目录权限失败：{}", dir.display()))?;
            }
            if self.cookies_file.exists() {
                fs::set_permissions(&self.cookies_file, fs::Permissions::from_mode(0o600))
                    .with_context(|| {
                        format!("收紧 Cookie 文件权限失败：{}", self.cookies_file.display())
                    })?;
            }
        }
        Ok(())
    }

    /// 打印标准目录结构
    pub fn print_std_dirs(&self) {
        println!("\n标准目录：");
        println!("  {}/", self.data_dir.display());
        println!("    ├── config/               — 账号配置");
        println!("    ├── cookies.json          — 账号 Cookie");
        println!("    ├── global_config.json    — 全局配置");
        println!("    └── logs/                 — 运行日志");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 权限收紧测试（仅 Unix，数据目录应仅所有者可访问） ──

    // 收紧后数据目录及子目录权限应为 0700
    #[cfg(unix)]
    #[test]
    fn test_tighten_permissions_dirs_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new_from(dir.path()).unwrap();
        paths.tighten_permissions().unwrap();

        for p in [paths.data_dir(), paths.logs_dir(), paths.config_dir()] {
            let mode = fs::metadata(p).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o700);
        }
    }

    // 已存在 Cookie 文件权限过宽（如老用户升级前的 0644），收紧后应为 0600
    #[cfg(unix)]
    #[test]
    fn test_tighten_permissions_existing_cookies_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new_from(dir.path()).unwrap();
        fs::write(paths.cookies_file(), "{}").unwrap();
        fs::set_permissions(paths.cookies_file(), fs::Permissions::from_mode(0o644)).unwrap();

        paths.tighten_permissions().unwrap();

        let mode = fs::metadata(paths.cookies_file())
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    // Cookie 文件不存在时收紧不应报错
    #[cfg(unix)]
    #[test]
    fn test_tighten_permissions_no_cookies_file_ok() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new_from(dir.path()).unwrap();
        assert!(!paths.cookies_file().exists());
        assert!(paths.tighten_permissions().is_ok());
    }
}
