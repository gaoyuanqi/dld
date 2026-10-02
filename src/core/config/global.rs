//! 全局配置：所有账号共享
//!
//! 配置文件：`<data_dir>/global_config.json`
//! `dld 同步配置` 会同步更新所有已登记账号的全局配置

use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use super::UpdatableConfig;

// ───────── 全局配置 ─────────

/// 全局配置（所有账号共享）
///
/// 配置文件：`<data_dir>/global_config.json`
///
/// `dld 同步配置` 会同步更新所有已登记账号的全局配置
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct GlobalConfig {
    pub 运行时: YunXingShi,
    pub 兑换码: DuiHuanMa,
    pub 时空遗迹: ShiKongYiJi,
}

impl UpdatableConfig for GlobalConfig {
    fn section_title() -> &'static str {
        "全局配置"
    }

    fn validate(&self) -> Result<()> {
        self.运行时.validate()?;
        self.兑换码.validate()?;
        self.时空遗迹.八卦迷阵.validate()?;
        Ok(())
    }
}

/// 运行时
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct YunXingShi {
    pub 并发数: u8,
    pub 日志保留天数: u8,
}

impl Default for YunXingShi {
    fn default() -> Self {
        Self {
            并发数: 5,
            日志保留天数: 30,
        }
    }
}

impl YunXingShi {
    fn validate(&self) -> Result<()> {
        validate_range!("运行时.并发数", self.并发数, 1, 20);
        validate_range!("运行时.日志保留天数", self.日志保留天数, 1, 90);
        Ok(())
    }
}

/// 兑换码
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DuiHuanMa {
    pub code: String,
}

impl Default for DuiHuanMa {
    fn default() -> Self {
        Self {
            code: "686866".to_string(),
        }
    }
}

impl DuiHuanMa {
    fn validate(&self) -> Result<()> {
        if self.code.len() != 6 || !self.code.chars().all(|c| c.is_ascii_digit()) {
            bail!("兑换码.code 数字字符串长度应为 6，实际为 \"{}\"", self.code);
        }
        Ok(())
    }
}

/// 时空遗迹
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ShiKongYiJi {
    pub 八卦迷阵: BaGuaMiZhen,
}

/// 八卦迷阵
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct BaGuaMiZhen {
    pub 第一层: BaGua,
    pub 第二层: BaGua,
    pub 第三层: BaGua,
    pub 第四层: BaGua,
}

impl Default for BaGuaMiZhen {
    fn default() -> Self {
        Self {
            第一层: BaGua::震,
            第二层: BaGua::巽,
            第三层: BaGua::坤,
            第四层: BaGua::离,
        }
    }
}

impl BaGuaMiZhen {
    /// 按层顺序返回卦象 id
    pub fn ids(&self) -> [u8; 4] {
        [
            self.第一层.id(),
            self.第二层.id(),
            self.第三层.id(),
            self.第四层.id(),
        ]
    }

    /// 从卦象字符串解析 id 序列，不足 4 个或含非法字符时返回 None
    pub fn chars_to_ids(&self, s: &str) -> Option<Vec<u8>> {
        let ids: Vec<u8> = s
            .chars()
            .filter_map(BaGua::from_char)
            .map(|b| b.id())
            .collect();
        if ids.len() == 4 { Some(ids) } else { None }
    }

    fn validate(&self) -> Result<()> {
        let ids = self.ids();
        let unique: HashSet<u8> = ids.iter().copied().collect();
        if unique.len() != 4 {
            bail!("时空遗迹.八卦迷阵 四层卦象不允许重复，当前：{:?}", ids);
        }
        Ok(())
    }
}

/// 八卦卦象
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaGua {
    离,
    坤,
    兑,
    乾,
    坎,
    艮,
    震,
    巽,
}

impl BaGua {
    /// 从字符解析卦象
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '离' => Some(BaGua::离),
            '坤' => Some(BaGua::坤),
            '兑' => Some(BaGua::兑),
            '乾' => Some(BaGua::乾),
            '坎' => Some(BaGua::坎),
            '艮' => Some(BaGua::艮),
            '震' => Some(BaGua::震),
            '巽' => Some(BaGua::巽),
            _ => None,
        }
    }

    /// 卦象对应的接口 id
    pub fn id(&self) -> u8 {
        match self {
            BaGua::离 => 1,
            BaGua::坤 => 2,
            BaGua::兑 => 3,
            BaGua::乾 => 4,
            BaGua::坎 => 5,
            BaGua::艮 => 6,
            BaGua::震 => 7,
            BaGua::巽 => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn config_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
        dir.path().join("global_config.json")
    }

    // 写入 JSON 后加载全局配置
    fn load_global(json: &str) -> Result<GlobalConfig> {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        fs::write(&path, json).unwrap();
        GlobalConfig::load(&path)
    }

    // ─── GlobalConfig 校验测试 ───

    // 默认值与空配置加载结果一致
    #[test]
    fn test_global_config_default_and_partial_load() {
        for config in [GlobalConfig::default(), load_global(r#"{}"#).unwrap()] {
            assert_eq!(config.运行时.并发数, 5);
            assert_eq!(config.运行时.日志保留天数, 30);
            assert_eq!(config.兑换码.code, "686866");
            assert_eq!(config.时空遗迹.八卦迷阵.第一层, BaGua::震);
            assert_eq!(config.时空遗迹.八卦迷阵.ids(), [7, 8, 2, 1]);
        }
    }

    // 运行时校验：边界值合法
    #[test]
    fn test_global_config_load_validate_range_ok() {
        let config = load_global(r#"{"运行时": {"并发数": 1, "日志保留天数": 90}}"#).unwrap();
        assert_eq!(config.运行时.并发数, 1);
        assert_eq!(config.运行时.日志保留天数, 90);
    }

    // 运行时超上限报错
    #[test]
    fn test_global_config_load_validate_out_of_range() {
        assert!(load_global(r#"{"运行时": {"并发数": 21}}"#).is_err());
        assert!(load_global(r#"{"运行时": {"日志保留天数": 91}}"#).is_err());
    }

    // update 路径校验：并发数为 0 报错
    #[test]
    fn test_global_config_update_validate_range_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        let json = r#"{"运行时": {"并发数": 0}}"#;
        fs::write(&path, json).unwrap();
        assert!(GlobalConfig::update(&path).is_err());
        assert_eq!(read_file(&path), json);
    }

    // 兑换码长度不足 6 位报错
    #[test]
    fn test_global_config_load_validate_code_too_short() {
        assert!(load_global(r#"{"兑换码": {"code": "12345"}}"#).is_err());
    }

    // 兑换码含非数字字符报错
    #[test]
    fn test_global_config_load_validate_code_not_digit() {
        assert!(load_global(r#"{"兑换码": {"code": "abc123"}}"#).is_err());
    }

    // update 路径校验：兑换码长度不足报错
    #[test]
    fn test_global_config_update_validate_code_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        let json = r#"{"兑换码": {"code": "12345"}}"#;
        fs::write(&path, json).unwrap();
        assert!(GlobalConfig::update(&path).is_err());
        assert_eq!(read_file(&path), json);
    }

    // ─── BaGua::from_char 测试 ───

    #[test]
    fn test_bagua_from_char_all_valid() {
        let chars = ['离', '坤', '兑', '乾', '坎', '艮', '震', '巽'];
        for &c in &chars {
            assert!(BaGua::from_char(c).is_some(), "字符 '{c}' 应解析成功");
        }
        assert_eq!(chars.len(), 8);
    }

    #[test]
    fn test_bagua_from_char_invalid() {
        assert!(BaGua::from_char('a').is_none());
        assert!(BaGua::from_char('1').is_none());
        assert!(BaGua::from_char('中').is_none());
        assert!(BaGua::from_char('雷').is_none());
    }

    // ─── BaGuaMiZhen::chars_to_ids 测试 ───

    #[test]
    fn test_chars_to_ids_valid() {
        let bagua = BaGuaMiZhen::default();
        // 乾=4 坤=2 坎=5 离=1
        let ids = bagua.chars_to_ids("乾坤坎离").unwrap();
        assert_eq!(ids, vec![4, 2, 5, 1]);
    }

    // 非法输入返回 None
    #[test]
    fn test_chars_to_ids_invalid() {
        let bagua = BaGuaMiZhen::default();
        // 不足 4 个
        assert!(bagua.chars_to_ids("乾坤").is_none());
        // 有效卦象超过 4 个
        assert!(bagua.chars_to_ids("乾坤坎离震").is_none());
        // 包含非法字符
        assert!(bagua.chars_to_ids("ab乾坤").is_none());
        // 空字符串
        assert!(bagua.chars_to_ids("").is_none());
    }

    // ─── BaGuaMiZhen::validate 测试 ───

    #[test]
    fn test_bagua_mizhen_validate_all_different() {
        let bagua = BaGuaMiZhen::default();
        // 默认配置「震巽坤离」四层皆不同
        assert!(bagua.validate().is_ok());
    }

    #[test]
    fn test_bagua_mizhen_validate_duplicate() {
        let bagua = BaGuaMiZhen {
            第一层: BaGua::震,
            第二层: BaGua::震,
            第三层: BaGua::坤,
            第四层: BaGua::离,
        };
        assert!(bagua.validate().is_err());
    }

    #[test]
    fn test_bagua_mizhen_validate_duplicate_via_load() {
        let json = r#"{"时空遗迹": {"八卦迷阵": {"第一层": "震", "第二层": "震", "第三层": "坤", "第四层": "离"}}}"#;
        assert!(load_global(json).is_err());
    }

    #[test]
    fn test_bagua_mizhen_validate_duplicate_via_update() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        let json = r#"{"时空遗迹": {"八卦迷阵": {"第一层": "震", "第二层": "震", "第三层": "坤", "第四层": "离"}}}"#;
        fs::write(&path, json).unwrap();
        assert!(GlobalConfig::update(&path).is_err());
        // 原文件不被破坏
        assert_eq!(read_file(&path), json);
    }
}
