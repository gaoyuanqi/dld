//! 配置管理：从 JSON 文件加载，不存在时自动创建默认
//!
//! # 两种配置
//!
//! - **全局配置**（`global_config.json`）：所有账号共享，如兑换码、八卦迷阵方向，见 `global` 模块
//! - **账号配置**（`config/<qq>.json`）：每 QQ 独享，如矿洞楼层、帮派商会物品，见 `account` 模块
//!
//! # 核心 trait
//!
//! `UpdatableConfig` 提供加载、校验、同步更新的统一接口
//! `dld 同步配置` 会对比磁盘 JSON 与默认结构体，自动补充新字段、删除废弃字段

/// 校验数值在闭区间 [min, max] 内，越界时报错
macro_rules! validate_range {
    ($name:expr, $value:expr, $min:expr, $max:expr) => {
        if !($min..=$max).contains(&$value) {
            bail!("{} 期望 {}~{}，实际为 {}", $name, $min, $max, $value);
        }
    };
}

mod account;
mod global;

pub use account::*;
pub use global::*;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

use serde::Serialize;
use serde_json::Value;

/// 同步配置 diff 结果
pub(crate) struct DiffResult {
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

/// 可更新配置文件的统一接口
///
/// 提供 [`load`](UpdatableConfig::load)、[`update`](UpdatableConfig::update)、
/// [`create_default`](UpdatableConfig::create_default) 等默认实现
/// 实现者只需提供 [`section_title`](UpdatableConfig::section_title)，
/// 可选覆盖 [`validate`](UpdatableConfig::validate) 校验字段取值范围
pub(crate) trait UpdatableConfig: Default + DeserializeOwned + Serialize {
    /// 配置节标题，如 "全局配置"、"账号配置"
    fn section_title() -> &'static str;

    /// 序列化并写入文件
    fn save(path: &Path, config: &Self) -> Result<()> {
        let json = serde_json::to_string_pretty(config)?;
        fs::write(path, json)
            .with_context(|| format!("写入{}失败：{}", Self::section_title(), path.display()))
    }

    /// 配置文件不存在时创建默认文件，已存在则跳过
    fn create_default(path: &Path) -> Result<()> {
        if !path.exists() {
            Self::save(path, &Self::default())?;
        }
        Ok(())
    }

    /// 加载后校验字段值，默认不校验
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// 加载配置，文件不存在则自动创建默认
    fn load(path: &Path) -> Result<Self> {
        Self::create_default(path)?;
        let content = fs::read_to_string(path)
            .with_context(|| format!("读取{}失败：{}", Self::section_title(), path.display()))?;
        let config: Self = serde_json::from_str(&content)
            .with_context(|| format!("{}格式错误：{}", Self::section_title(), path.display()))?;
        config
            .validate()
            .with_context(|| format!("{}校验失败：{}", Self::section_title(), path.display()))?;
        Ok(config)
    }

    /// 合并更新配置文件：补充新增字段、删除废弃字段，保留用户已有值
    ///
    /// 流程：读旧 JSON → 与默认结构体 diff → 用 `#[serde(default)]` 反序列化合并
    /// → 序列化写回 → 返回 diff 结果
    fn update(path: &Path) -> Result<DiffResult> {
        // 1. 读取磁盘上的旧 JSON（不存在则用空对象）
        let old_value = if path.exists() {
            let content = fs::read_to_string(path).with_context(|| {
                format!("读取{}失败：{}", Self::section_title(), path.display())
            })?;
            serde_json::from_str(&content)
                .with_context(|| format!("{}格式错误：{}", Self::section_title(), path.display()))?
        } else {
            Value::Object(serde_json::Map::new())
        };

        // 2. 序列化默认结构体为 JSON，与旧值 diff
        let default_value = serde_json::to_value(Self::default())?;
        let diff = diff_fields(&old_value, &default_value);

        // 3. 用 #[serde(default)] 反序列化：旧值中缺失的字段自动补默认值
        let label = if old_value.as_object().map(|m| m.is_empty()).unwrap_or(false) {
            String::from("空文件")
        } else {
            path.display().to_string()
        };
        let config: Self = serde_json::from_value(old_value)
            .with_context(|| format!("{}类型错误：{label}", Self::section_title()))?;

        // 3.5 校验字段取值合法性
        config
            .validate()
            .with_context(|| format!("{}校验失败：{}", Self::section_title(), path.display()))?;

        // 4. 无变化则跳过写入
        if diff.added.is_empty() && diff.removed.is_empty() {
            return Ok(diff);
        }

        // 5. 序列化写回，废弃字段被自动剔除
        Self::save(path, &config)?;

        Ok(diff)
    }

    /// 更新并打印 diff 报告
    fn update_and_report(path: &Path) -> Result<()> {
        let default_value = serde_json::to_value(Self::default())?;
        let diff = Self::update(path)?;
        let has_changes = !diff.added.is_empty() || !diff.removed.is_empty();

        if !diff.added.is_empty() {
            println!("\n新增：");
            for p in &diff.added {
                if let Some(v) = get_leaf_value(&default_value, p) {
                    println!("  {p}: {}，默认 {}", value_kind(v), format_default(v));
                }
            }
        }
        if !diff.removed.is_empty() {
            println!("\n移除：");
            for r in &diff.removed {
                println!("  {r}");
            }
        }
        if has_changes {
            println!("\n{}已更新：{}", Self::section_title(), path.display());
        }

        Ok(())
    }
}

/// JSON 值类型的中文名
fn value_kind(v: &Value) -> &'static str {
    match v {
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
}

/// 格式化默认值（字符串加引号）
fn format_default(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{s}\""),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => {
            let len = arr.len();
            if len == 0 {
                "[]".to_string()
            } else {
                format!("[{len} 个元素]")
            }
        }
        Value::Object(_) => "{}".to_string(),
    }
}

/// 从 JSON Value 中按点号分隔路径获取叶子值
fn get_leaf_value<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

/// 提取 JSON 对象中所有叶子字段的路径（用 "." 分隔）
fn leaf_paths(value: &Value, prefix: &str) -> Vec<String> {
    match value {
        Value::Object(map) => {
            let mut paths = Vec::new();
            for (k, v) in map {
                let full_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                paths.extend(leaf_paths(v, &full_key));
            }
            paths
        }
        _ => vec![prefix.to_string()],
    }
}

/// 对比旧 JSON 与默认结构体 JSON
fn diff_fields(old_value: &Value, default_value: &Value) -> DiffResult {
    let old_paths: HashSet<String> = leaf_paths(old_value, "").into_iter().collect();
    let default_paths: HashSet<String> = leaf_paths(default_value, "").into_iter().collect();

    let mut added: Vec<String> = default_paths.difference(&old_paths).cloned().collect();
    let mut removed: Vec<String> = old_paths.difference(&default_paths).cloned().collect();
    added.sort();
    removed.sort();

    DiffResult { added, removed }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    // ─── UpdatableConfig load/update 流程测试（以 GlobalConfig 为载体）───

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

    // 子对象存在但内部为空，用默认值补齐
    #[test]
    fn test_load_empty_sub_object_fills_defaults() {
        let config = load_global(r#"{"兑换码": {}}"#).unwrap();
        assert_eq!(config.兑换码.code, "686866");
    }

    // 已有合法配置文件，读取后保留用户值
    #[test]
    fn test_load_existing_valid_file() {
        let config = load_global(r#"{"兑换码": {"code": "888888"}}"#).unwrap();
        assert_eq!(config.兑换码.code, "888888");
    }

    // 配置文件不存在时自动创建默认文件
    #[test]
    fn test_load_creates_default_when_not_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        assert!(!path.exists());
        let config = GlobalConfig::load(&path).unwrap();
        assert_eq!(config.兑换码.code, "686866");
        // load 应自动创建文件
        assert!(path.exists());
    }

    // 非法 JSON 应报错
    #[test]
    fn test_load_invalid_json_errors() {
        assert!(load_global("not json").is_err());
    }

    // 字段类型不匹配应报错
    #[test]
    fn test_load_type_mismatch_errors() {
        // code 是 String，给数字应报错
        assert!(load_global(r#"{"兑换码": {"code": 123}}"#).is_err());
    }

    // 文件不存在时自动创建
    #[test]
    fn test_update_file_not_exists_creates_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        let diff = GlobalConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(content.contains("686866"));
        assert!(content.contains(r#""第一层""#));
        // 空对象 → 所有字段都是新增
        assert!(!diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    // 无新增无废弃时不写入
    #[test]
    fn test_update_no_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        let json = serde_json::to_string_pretty(&GlobalConfig::default()).unwrap();
        fs::write(&path, &json).unwrap();
        let before = read_file(&path);
        let diff = GlobalConfig::update(&path).unwrap();
        // 无变化时 diff 为空
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        // 文件内容不变（去除空白差异）
        let after = read_file(&path);
        assert_eq!(before.trim(), after.trim());
    }

    // 新增缺失字段，保留已有值
    #[test]
    fn test_update_adds_missing_fields_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        // 只写兑换码，缺少时空遗迹
        let json = r#"{"兑换码": {"code": "888888"}}"#;
        fs::write(&path, json).unwrap();
        let diff = GlobalConfig::update(&path).unwrap();
        let content = read_file(&path);
        // 已有值保留
        assert!(content.contains("888888"));
        // 新增字段补上
        assert!(content.contains(r#""第一层""#));
        // diff 报告中应有新增、无移除
        assert!(!diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    // 子对象存在但内部为空，用默认值补齐
    #[test]
    fn test_update_empty_sub_object_fills_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        // 子对象存在但内部字段为空
        let json = r#"{"兑换码": {}}"#;
        fs::write(&path, json).unwrap();
        let diff = GlobalConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(content.contains("686866"));
        assert!(!diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    // 废弃字段被移除，已有值保留
    #[test]
    fn test_update_removes_deprecated_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        // 包含一个不存在的旧字段
        let json = r#"{"兑换码": {"code": "686866"}, "废弃项": {"x": "y"}}"#;
        fs::write(&path, json).unwrap();
        let diff = GlobalConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(!content.contains("废弃项"));
        assert!(content.contains("686866"));
        // diff 报告应包含移除的废弃字段
        assert!(diff.removed.iter().any(|r| r.starts_with("废弃项")));
    }

    // 空文件 {} 自动补全所有字段
    #[test]
    fn test_update_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        fs::write(&path, "{}").unwrap();
        GlobalConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(content.contains("686866"));
        assert!(content.contains(r#""第一层""#));
    }

    // 非法 JSON 报错且不破坏原文件
    #[test]
    fn test_update_invalid_json_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        let junk = "not json";
        fs::write(&path, junk).unwrap();
        assert!(GlobalConfig::update(&path).is_err());
        // 文件不被改动
        assert_eq!(read_file(&path), junk);
    }

    // 字段类型不匹配报错且不破坏原文件
    #[test]
    fn test_update_type_mismatch_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        let json = r#"{"兑换码": {"code": 123}}"#;
        fs::write(&path, json).unwrap();
        assert!(GlobalConfig::update(&path).is_err());
        // 文件不被改动
        assert_eq!(read_file(&path), json);
    }
}
