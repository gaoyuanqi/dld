//! 账号配置：每个 QQ 独享
//!
//! 配置文件：`<data_dir>/config/<qq>.json`
//! 各字段为对应玩法的配置开关和参数

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{
    BangPaiShangHui, BeiBao, DengLuShangDian, HuiWu, JiLiDui, JiangHuChangMeng, JingJiChang,
    KaiXueJi, KuangDong, LiLian, LongHuangZhiJing, LueDuo, MenPai, MenPaiYaoQingSai,
    MengXiangZhiLv, ShenYuanZhiChao, ShiKongYiJiAccount, UpdatableConfig, WenDingTianXia,
    WoDeBangPai, XiaShiKeZhan,
};

// ───────── 账号配置 ─────────

/// 账号配置（每个 QQ 独享）
///
/// 配置文件：`<data_dir>/config/<qq>.json`
///
/// 各字段为对应玩法的配置开关和参数
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct AccountConfig {
    pub 矿洞: KuangDong,
    pub 掠夺: LueDuo,
    pub 历练: LiLian,
    pub 门派: MenPai,
    pub 会武: HuiWu,
    pub 背包: BeiBao,
    pub 竞技场: JingJiChang,
    pub 梦想之旅: MengXiangZhiLv,
    pub 问鼎天下: WenDingTianXia,
    pub 帮派商会: BangPaiShangHui,
    pub 侠士客栈: XiaShiKeZhan,
    pub 江湖长梦: JiangHuChangMeng,
    pub 深渊之潮: ShenYuanZhiChao,
    pub 龙凰之境: LongHuangZhiJing,
    pub 我的帮派: WoDeBangPai,
    pub 时空遗迹: ShiKongYiJiAccount,
    pub 门派邀请赛: MenPaiYaoQingSai,
    pub 开学季: KaiXueJi,
    pub 吉利兑: JiLiDui,
    pub 登录商店: DengLuShangDian,
}

impl UpdatableConfig for AccountConfig {
    fn section_title() -> &'static str {
        "账号配置"
    }

    fn validate(&self) -> Result<()> {
        self.掠夺.validate()?;
        self.历练.validate()?;
        self.会武.validate()?;
        self.背包.validate()?;
        self.梦想之旅.validate()?;
        self.帮派商会.兑换商店.validate()?;
        self.江湖长梦.兑换上限.validate()?;
        self.江湖长梦.副本.validate()?;
        self.龙凰之境.兑换上限.validate()?;
        self.我的帮派.validate()?;
        self.门派邀请赛.兑换.validate()?;
        self.吉利兑.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::core::config::{KuangDongFloor, KuangDongMode, LiLianBoss, YiShouDongKuBoss};

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    // 账号配置路径
    fn account_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
        dir.path().join("config.json")
    }

    // 写入 JSON 后加载账号配置
    fn load_account(json: &str) -> Result<AccountConfig> {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        fs::write(&path, json).unwrap();
        AccountConfig::load(&path)
    }

    // ─── AccountConfig load 测试 ───

    // 默认值正确
    #[test]
    fn test_account_config_default() {
        let config = AccountConfig::default();
        assert_eq!(config.矿洞.开启副本.层数, KuangDongFloor::F1);
        assert_eq!(config.矿洞.开启副本.模式, KuangDongMode::Easy);
        assert_eq!(
            config.背包.使用,
            [
                "周年福袋",
                "福运红包",
                "财运红包",
                "阅历卷宗",
                "贡献小笼包",
                "贡献叉烧包",
                "阅历羊皮卷",
                "装酒肉的食盒",
            ]
        );
        assert!(!config.竞技场.兑换河图洛书);
        assert_eq!(config.梦想之旅.最多消耗梦幻机票数量, 3);
        assert!(!config.侠士客栈.黑市商人.ex_1006);
        assert!(!config.问鼎天下.兑换碎片);
        assert_eq!(config.历练.乐斗顺序.len(), 9);
        assert_eq!(config.历练.乐斗顺序[0], LiLianBoss::XiongShi);
        assert_eq!(config.历练.乐斗顺序[8], LiLianBoss::XiangXian);
        assert_eq!(
            config.时空遗迹.异兽洞窟.max_boss,
            YiShouDongKuBoss::王级异兽母巢
        );
    }

    // 已有合法配置文件，读取后保留用户值
    #[test]
    fn test_account_config_load_existing_valid() {
        let json = r#"{"矿洞": {"开启副本": {"层数": "第三层", "模式": "普通"}}}"#;
        let config = load_account(json).unwrap();
        assert_eq!(config.矿洞.开启副本.层数, KuangDongFloor::F3);
        assert_eq!(config.矿洞.开启副本.模式, KuangDongMode::Normal);
    }

    // 配置文件不存在时自动创建默认文件
    #[test]
    fn test_account_config_load_creates_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        assert!(!path.exists());
        let config = AccountConfig::load(&path).unwrap();
        assert_eq!(config.矿洞.开启副本.层数, KuangDongFloor::F1);
        // load 应自动创建文件
        assert!(path.exists());
    }

    // 部分字段缺失时用默认值补齐
    #[test]
    fn test_account_config_load_partial_file_fills_defaults() {
        let config = load_account(r#"{}"#).unwrap();
        assert_eq!(config.矿洞.开启副本.层数, KuangDongFloor::F1);
        assert_eq!(config.矿洞.开启副本.模式, KuangDongMode::Easy);
        assert!(!config.竞技场.兑换河图洛书);
    }

    // 子对象存在但内部为空，用默认值补齐
    #[test]
    fn test_account_config_load_empty_sub_object_fills_defaults() {
        let config = load_account(r#"{"竞技场": {}}"#).unwrap();
        assert!(!config.竞技场.兑换河图洛书);
    }

    // 子配置 validate 已挂到 AccountConfig（走完整加载链路）
    #[test]
    fn test_account_config_load_validate_range() {
        assert!(load_account(r#"{"梦想之旅": {"最多消耗梦幻机票数量": 10}}"#).is_err());
    }

    // 非法 JSON 应报错
    #[test]
    fn test_account_config_load_invalid_json_errors() {
        assert!(load_account("not json").is_err());
    }

    // 枚举值无效应报错
    #[test]
    fn test_account_config_load_type_mismatch_errors() {
        // 层数 不是有效枚举值应报错
        assert!(load_account(r#"{"矿洞": {"开启副本": {"层数": "无效层"}}}"#).is_err());
    }

    // ─── AccountConfig update 测试 ───

    // 文件不存在时自动创建
    #[test]
    fn test_account_config_update_file_not_exists_creates_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let diff = AccountConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(content.contains("第一层"));
        assert!(content.contains("简单"));
        assert!(content.contains("兑换河图洛书"));
        assert!(!diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    // 无新增无废弃时不写入
    #[test]
    fn test_account_config_update_no_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let json = serde_json::to_string_pretty(&AccountConfig::default()).unwrap();
        fs::write(&path, &json).unwrap();
        let before = read_file(&path);
        let diff = AccountConfig::update(&path).unwrap();
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        let after = read_file(&path);
        assert_eq!(before.trim(), after.trim());
    }

    // 新增缺失字段，保留已有值
    #[test]
    fn test_account_config_update_adds_missing_fields_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let json = r#"{"矿洞": {"开启副本": {"层数": "第五层"}}}"#;
        fs::write(&path, json).unwrap();
        let diff = AccountConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(content.contains("第五层"));
        assert!(content.contains("简单"));
        assert!(!diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    // 子对象存在但内部为空，用默认值补齐
    #[test]
    fn test_account_config_update_empty_sub_object_fills_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let json = r#"{"竞技场": {}}"#;
        fs::write(&path, json).unwrap();
        let diff = AccountConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(content.contains("兑换河图洛书"));
        assert!(!diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    // 废弃字段被移除，已有值保留
    #[test]
    fn test_account_config_update_removes_deprecated_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let json = r#"{"矿洞": {"开启副本": {"层数": "第一层"}}, "废弃项": {"x": "y"}}"#;
        fs::write(&path, json).unwrap();
        let diff = AccountConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(!content.contains("废弃项"));
        assert!(content.contains("第一层"));
        assert!(diff.removed.iter().any(|r| r.starts_with("废弃项")));
    }

    // 空文件 {} 自动补全所有字段
    #[test]
    fn test_account_config_update_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        fs::write(&path, "{}").unwrap();
        AccountConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(content.contains("第一层"));
        assert!(content.contains("简单"));
    }

    // 非法 JSON 报错且不破坏原文件
    #[test]
    fn test_account_config_update_invalid_json_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let junk = "not json";
        fs::write(&path, junk).unwrap();
        assert!(AccountConfig::update(&path).is_err());
        assert_eq!(read_file(&path), junk);
    }

    // 枚举值无效应报错且不破坏原文件
    #[test]
    fn test_account_config_update_type_mismatch_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let json = r#"{"矿洞": {"开启副本": {"层数": "无效层"}}}"#;
        fs::write(&path, json).unwrap();
        assert!(AccountConfig::update(&path).is_err());
        assert_eq!(read_file(&path), json);
    }

    // 子配置 validate 挂接有效：校验失败报错且不破坏原文件
    #[test]
    fn test_account_config_update_range_validation_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let json = r#"{"梦想之旅": {"最多消耗梦幻机票数量": 50}}"#;
        fs::write(&path, json).unwrap();
        assert!(AccountConfig::update(&path).is_err());
        assert_eq!(read_file(&path), json);
    }
}
