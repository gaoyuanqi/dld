//! 大乐斗账号持久化管理：Cookie 解析、登记、注销、查询
//!
//! Cookie 格式为 `openId=xxx; accessToken=yyy; newuin=<QQ>`

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// 配置文件中的单条记录（`#[serde(default)]` 保证未来字段兼容）
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AccountRecord {
    pub cookie: String,
}

/// 单个账号（QQ + 配置）
#[derive(Clone, Debug)]
pub struct Account {
    pub qq: String,
    pub record: AccountRecord,
}

impl Account {
    /// 从原始 Cookie 字符串构建账号（登记用）
    pub fn from_cookie(raw: &str) -> Result<Self> {
        let cookie = Self::format_cookie(raw)?;
        let qq = Self::extract_qq(&cookie)?;
        Ok(Self {
            qq,
            record: AccountRecord { cookie },
        })
    }

    /// 格式化为标准形式：`openId=...; accessToken=...; newuin=...`
    fn format_cookie(cookie: &str) -> Result<String> {
        let mut open_id = None;
        let mut access_token = None;
        let mut new_uin = None;

        for (key, value) in Self::parse_cookie_parts(cookie) {
            match key {
                "openId" => open_id = Some(value),
                "accessToken" => access_token = Some(value),
                "newuin" => new_uin = Some(value),
                _ => {}
            }
        }

        let (Some(o), Some(a), Some(n)) = (open_id, access_token, new_uin) else {
            bail!("Cookie 必须包含 openId、accessToken、newuin 三个字段");
        };

        Ok(format!("openId={o}; accessToken={a}; newuin={n}"))
    }

    /// 从已格式化的 Cookie 中提取 QQ 号（即 newuin 字段的值）
    fn extract_qq(cookie: &str) -> Result<String> {
        for (key, value) in Self::parse_cookie_parts(cookie) {
            if key == "newuin" {
                if !value.chars().all(|c| c.is_ascii_digit()) {
                    bail!("newuin 必须为纯数字，当前值: {}", value);
                }
                return Ok(value.to_string());
            }
        }
        bail!("未找到 newuin 字段")
    }

    /// 解析 Cookie 字符串，返回键值对迭代器（忽略空值字段）
    fn parse_cookie_parts(cookie: &str) -> impl Iterator<Item = (&str, &str)> + '_ {
        cookie.split(';').filter_map(|part| {
            let part = part.trim();
            let eq_pos = part.find('=')?;
            let key = part[..eq_pos].trim();
            let value = part[eq_pos + 1..].trim();
            if value.is_empty() {
                return None;
            }
            Some((key, value))
        })
    }
}

/// 账号持久化存储
pub struct AccountStore {
    path: PathBuf,
}

impl AccountStore {
    /// 创建账号存储（指定 Cookie 文件路径）
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// 将账号 Cookie 写入文件
    pub fn save(&self, account: &Account) -> Result<()> {
        let mut accounts = self.load_cookies()?;

        accounts
            .entry(account.qq.clone())
            .and_modify(|record| *record = account.record.clone())
            .or_insert(account.record.clone());

        let json = serde_json::to_string_pretty(&accounts).context("序列化账号数据失败")?;
        fs::write(&self.path, json)
            .with_context(|| format!("写入账号文件失败：{}", self.path.display()))?;

        println!("\n{} 登记成功！", account.qq);

        Ok(())
    }

    /// 删除指定 QQ 的账号记录
    pub fn remove(&self, qq: &str) -> Result<()> {
        let mut accounts = self.load_cookies()?;

        if accounts.remove(qq).is_some() {
            let json = serde_json::to_string_pretty(&accounts).context("序列化账号数据失败")?;
            fs::write(&self.path, json)
                .with_context(|| format!("写入账号文件失败：{}", self.path.display()))?;
            println!("\n{} 注销成功！", qq);
        } else {
            println!("\n未找到 QQ {} 的账号，无需移除", qq);
        }

        Ok(())
    }

    /// 加载所有账号（保证至少有一个）
    pub fn load_all(&self) -> Result<Vec<Account>> {
        let accounts: Vec<_> = self
            .load_cookies()?
            .into_iter()
            .map(|(qq, record)| Account { qq, record })
            .collect();
        if accounts.is_empty() {
            bail!("没有已登记的账号，请先使用「登记」命令添加 Cookie");
        }
        Ok(accounts)
    }

    /// 返回所有已登记账号的 QQ 号列表
    pub fn list_qqs(&self) -> Result<Vec<String>> {
        let qqs: Vec<String> = self.load_cookies()?.into_keys().collect();
        Ok(qqs)
    }

    /// 按 QQ 号查找单个账号
    pub fn find(&self, qq: &str) -> Result<Account> {
        let accounts = self.load_cookies()?;
        let Some(record) = accounts.get(qq) else {
            bail!("未找到 QQ 号为 {} 的账号", qq)
        };
        Ok(Account {
            qq: qq.to_string(),
            record: record.clone(),
        })
    }

    fn load_cookies(&self) -> Result<HashMap<String, AccountRecord>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("读取账号文件失败：{}", self.path.display()))?;
        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }

        let accounts: HashMap<String, AccountRecord> = serde_json::from_str(&content)
            .with_context(|| format!("账号文件格式错误：{}", self.path.display()))?;
        Ok(accounts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cookie() {
        // 基本格式化
        assert_eq!(
            Account::format_cookie("openId=user123; accessToken=tokenABC; newuin=123").unwrap(),
            "openId=user123; accessToken=tokenABC; newuin=123"
        );
        // 过滤多余字段
        assert_eq!(
            Account::format_cookie(
                "openId=foo; accessToken=bar; newuin=123; expires=Wed, 21 Oct 2026 07:28:00 GMT"
            )
            .unwrap(),
            "openId=foo; accessToken=bar; newuin=123"
        );
        // 容错空格
        assert_eq!(
            Account::format_cookie(" openId = value1 ; accessToken = value2 ; newuin = 123 ")
                .unwrap(),
            "openId=value1; accessToken=value2; newuin=123"
        );
    }

    #[test]
    fn test_format_cookie_missing_field() {
        let cookie = "accessToken=abc; newuin=123";
        assert!(Account::format_cookie(cookie).is_err());
        let cookie = "openId=123; newuin=123";
        assert!(Account::format_cookie(cookie).is_err());
        let cookie = "openId=123; accessToken=abc";
        assert!(Account::format_cookie(cookie).is_err());
    }

    #[test]
    fn test_format_cookie_empty_value() {
        assert!(Account::format_cookie("openId=; accessToken=abc; newuin=123").is_err());
        assert!(Account::format_cookie("openId=abc; accessToken=; newuin=123").is_err());
        assert!(Account::format_cookie("openId=abc; accessToken=bar; newuin=").is_err());
    }

    #[test]
    fn test_extract_qq() {
        let cookie = "openId=user123; accessToken=tokenABC; newuin=10001";
        assert_eq!(Account::extract_qq(cookie).unwrap(), "10001");
    }

    #[test]
    fn test_extract_qq_missing() {
        assert!(Account::extract_qq("openId=user123; accessToken=tokenABC").is_err());
    }

    #[test]
    fn test_extract_qq_empty_value() {
        assert!(Account::extract_qq("openId=user123; accessToken=tokenABC; newuin=").is_err());
    }

    #[test]
    fn test_extract_qq_non_numeric() {
        assert!(
            Account::extract_qq("openId=user123; accessToken=tokenABC; newuin=abc123").is_err()
        );
    }

    // ── AccountStore 测试 ──

    fn sample_account(qq: &str) -> Account {
        Account {
            qq: qq.into(),
            record: AccountRecord {
                cookie: format!("openId=x; accessToken=y; newuin={qq}"),
            },
        }
    }

    #[test]
    fn test_save_and_load_all() {
        let (store, _dir) = temp_store();
        store.save(&sample_account("10001")).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].qq, "10001");
        assert_eq!(
            loaded[0].record.cookie,
            "openId=x; accessToken=y; newuin=10001"
        );
    }

    #[test]
    fn test_save_overwrites_existing() {
        let (store, _dir) = temp_store();
        store.save(&sample_account("10001")).unwrap();

        let updated = Account {
            qq: "10001".into(),
            record: AccountRecord {
                cookie: "openId=new; accessToken=new; newuin=10001".into(),
            },
        };
        store.save(&updated).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(
            loaded[0].record.cookie,
            "openId=new; accessToken=new; newuin=10001"
        );
    }

    #[test]
    fn test_list_qqs_multiple() {
        let (store, _dir) = temp_store();
        store.save(&sample_account("10001")).unwrap();
        store.save(&sample_account("20002")).unwrap();

        let mut qqs = store.list_qqs().unwrap();
        qqs.sort();
        assert_eq!(qqs, vec!["10001", "20002"]);
    }

    #[test]
    fn test_load_all_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("nonexistent.json"));
        assert!(store.load_all().is_err());
    }

    #[test]
    fn test_find_success() {
        let (store, _dir) = temp_store();
        store.save(&sample_account("10001")).unwrap();

        let found = store.find("10001").unwrap();
        assert_eq!(found.qq, "10001");
        assert_eq!(found.record.cookie, "openId=x; accessToken=y; newuin=10001");
    }

    #[test]
    fn test_find_not_found() {
        let (store, _dir) = temp_store();
        store.save(&sample_account("10001")).unwrap();

        assert!(store.find("99999").is_err());
    }

    #[test]
    fn test_remove_success() {
        let (store, _dir) = temp_store();
        store.save(&sample_account("10001")).unwrap();
        store.remove("10001").unwrap();

        assert!(store.load_all().is_err());
    }

    #[test]
    fn test_remove_nonexistent() {
        let (store, _dir) = temp_store();
        // 不保存任何账号，直接删除不存在的 — 不应报错
        assert!(store.remove("99999").is_ok());
    }

    fn temp_store() -> (AccountStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        (store, dir)
    }
}
