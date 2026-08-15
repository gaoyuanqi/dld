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
        let data_dir = dirs.data_local_dir();

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
