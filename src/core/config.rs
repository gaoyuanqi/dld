//! 配置管理：从 JSON 文件加载，不存在时自动创建默认
//!
//! # 两种配置
//!
//! - **全局配置**（`global_config.json`）：所有账号共享，如兑换码、八卦迷阵方向
//! - **账号配置**（`config/<qq>.json`）：每 QQ 独享，如矿洞楼层、帮派商会物品
//!
//! # 核心 trait
//!
//! `UpdatableConfig` 提供加载、校验、同步更新的统一接口
//! `dld 同步配置` 会对比磁盘 JSON 与默认结构体，自动补充新字段、删除废弃字段
//!
//! 全局配置文件：`<data_dir>/global_config.json`

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;

use serde::{Deserialize, Serialize};
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

/// 校验数值在闭区间 [min, max] 内，越界时报错
macro_rules! validate_range {
    ($name:expr, $value:expr, $min:expr, $max:expr) => {
        if !($min..=$max).contains(&$value) {
            bail!("{} 期望 {}~{}，实际为 {}", $name, $min, $max, $value);
        }
    };
}

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
            code: "161616".to_string(),
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
    pub 竞技场: JingJiChang,
    pub 梦想之旅: MengXiangZhiLv,
    pub 问鼎天下: WenDingTianXia,
    pub 帮派商会: BangPaiShangHui,
    pub 侠士客栈: XiaShiKeZhan,
    pub 江湖长梦: JiangHuChangMeng,
    pub 深渊之潮: ShenYuanZhiChao,
    pub 龙凰之境: LongHuangZhiJing,
    pub 我的帮派: WoDeBangPai,
    pub 门派邀请赛: MenPaiYaoQingSai,
}

impl UpdatableConfig for AccountConfig {
    fn section_title() -> &'static str {
        "账号配置"
    }

    fn validate(&self) -> Result<()> {
        self.掠夺.validate()?;
        self.历练.validate()?;
        self.会武.validate()?;
        self.梦想之旅.validate()?;
        self.帮派商会.兑换商店.validate()?;
        self.江湖长梦.兑换上限.validate()?;
        self.江湖长梦.副本.validate()?;
        self.龙凰之境.兑换上限.validate()?;
        self.我的帮派.validate()?;
        self.门派邀请赛.兑换.validate()?;
        Ok(())
    }
}

/// 矿洞
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct KuangDong {
    pub 开启副本: KuangDongFuBen,
}

/// 矿洞副本
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct KuangDongFuBen {
    /// 第一~第五层
    pub 层数: KuangDongFloor,
    /// 简单、普通、困难
    pub 模式: KuangDongMode,
}

/// 矿洞层数
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum KuangDongFloor {
    #[serde(rename = "第一层")]
    F1,
    #[serde(rename = "第二层")]
    F2,
    #[serde(rename = "第三层")]
    F3,
    #[serde(rename = "第四层")]
    F4,
    #[serde(rename = "第五层")]
    F5,
}

impl KuangDongFloor {
    pub(crate) fn api_value(&self) -> &str {
        match self {
            Self::F1 => "1",
            Self::F2 => "2",
            Self::F3 => "3",
            Self::F4 => "4",
            Self::F5 => "5",
        }
    }
}

/// 矿洞模式
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum KuangDongMode {
    #[serde(rename = "简单")]
    Easy,
    #[serde(rename = "普通")]
    Normal,
    #[serde(rename = "困难")]
    Hard,
}

impl KuangDongMode {
    pub(crate) fn api_value(&self) -> &str {
        match self {
            Self::Easy => "1",
            Self::Normal => "2",
            Self::Hard => "3",
        }
    }
}

impl Default for KuangDongFuBen {
    fn default() -> Self {
        Self {
            层数: KuangDongFloor::F1,
            模式: KuangDongMode::Easy,
        }
    }
}

/// 掠夺
///
/// | 目标战力 | 战力增量 | 行为 |
/// |---|---|------|
/// | >0 | >0 | 从指定门槛开始，找不到逐步升高 |
/// | >0 | =0 | 只打门槛以下的，找不到就停 |
/// | =0 | >0 | 从 0 开始自动递增 |
/// | =0 | =0 | 跳过掠夺 |
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LueDuo {
    /// 筛选对手的起始战力阈值
    pub 目标战力: u32,
    /// 每轮递增步长，设为 0 则不递增
    pub 战力增量: u32,
}

impl Default for LueDuo {
    fn default() -> Self {
        Self {
            目标战力: 1000,
            战力增量: 0,
        }
    }
}

impl LueDuo {
    fn validate(&self) -> Result<()> {
        validate_range!("掠夺.目标战力", self.目标战力, 0, 99999);
        validate_range!("掠夺.战力增量", self.战力增量, 0, 9999);
        Ok(())
    }
}

/// 历练
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LiLian {
    /// BOSS 乐斗顺序，必须包含全部 9 个且不重复
    pub 乐斗顺序: Vec<LiLianBoss>,
}

impl Default for LiLian {
    fn default() -> Self {
        Self {
            乐斗顺序: vec![
                LiLianBoss::XiongShi,
                LiLianBoss::XiaBingTouMu,
                LiLianBoss::YeChaYuanShuai,
                LiLianBoss::PiLiTouLing,
                LiLianBoss::SongJiang,
                LiLianBoss::DaPeng,
                LiLianBoss::MaDaWang,
                LiLianBoss::ShiXueGuiWang,
                LiLianBoss::XiangXian,
            ],
        }
    }
}

impl LiLian {
    fn validate(&self) -> Result<()> {
        let bosses = &self.乐斗顺序;
        if bosses.len() != 9 {
            bail!(
                "历练.乐斗顺序 必须包含全部 9 个 BOSS，当前 {} 个",
                bosses.len()
            );
        }
        let unique: HashSet<_> = bosses.iter().collect();
        if unique.len() != bosses.len() {
            bail!("历练.乐斗顺序 不允许重复");
        }
        Ok(())
    }
}

/// 历练 BOSS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum LiLianBoss {
    #[serde(rename = "凶尸-令狐冲")]
    XiongShi,
    #[serde(rename = "虾兵头目-丁春秋")]
    XiaBingTouMu,
    #[serde(rename = "夜叉元帅-丘处机")]
    YeChaYuanShuai,
    #[serde(rename = "霹雳头领-小龙女")]
    PiLiTouLing,
    #[serde(rename = "宋姜-韦小宝")]
    SongJiang,
    #[serde(rename = "大鹏-扫地僧")]
    DaPeng,
    #[serde(rename = "马大王-鹤笔翁")]
    MaDaWang,
    #[serde(rename = "嗜血鬼王-韦一笑")]
    ShiXueGuiWang,
    #[serde(rename = "象仙-赵敏")]
    XiangXian,
}

impl LiLianBoss {
    /// 所属场景 MapID
    pub fn mapid(self) -> &'static str {
        match self {
            Self::XiongShi => "2",
            Self::XiaBingTouMu => "3",
            Self::YeChaYuanShuai => "4",
            Self::PiLiTouLing => "5",
            Self::SongJiang => "6",
            Self::DaPeng => "7",
            Self::MaDaWang => "8",
            Self::ShiXueGuiWang => "9",
            Self::XiangXian => "10",
        }
    }
}

/// 门派
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MenPai {
    pub 万年寺: WanNianSi,
    pub 八叶堂: BaYeTang,
}

/// 万年寺
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct WanNianSi {
    /// 获得门贡*40，门派强化书*1
    pub 付费高香香炉: bool,
}

/// 八叶堂
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct BaYeTang {
    /// 除了获得门贡*60，还有3点活跃度
    pub 付费同门切磋: bool,
}

/// 会武
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct HuiWu {
    /// 成功助威后兑换真黄金卷轴数量，0~100
    pub 兑换真黄金卷轴数量: u32,
}

impl HuiWu {
    fn validate(&self) -> Result<()> {
        validate_range!("会武.兑换真黄金卷轴数量", self.兑换真黄金卷轴数量, 0, 100);
        Ok(())
    }
}

/// 竞技场
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JingJiChang {
    /// 是否赛季期每天兑换十个
    pub 兑换河图洛书: bool,
}

/// 梦想之旅
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MengXiangZhiLv {
    /// 周四消耗梦幻机票数量， 0~9
    pub 最多消耗梦幻机票数量: u32,
}

impl Default for MengXiangZhiLv {
    fn default() -> Self {
        Self {
            最多消耗梦幻机票数量: 3,
        }
    }
}

impl MengXiangZhiLv {
    fn validate(&self) -> Result<()> {
        validate_range!(
            "梦想之旅.最多消耗梦幻机票数量",
            self.最多消耗梦幻机票数量,
            0,
            9
        );
        Ok(())
    }
}

/// 问鼎天下
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct WenDingTianXia {
    /// 攻占资源点区域
    pub 攻占区域: Region,
    /// 付费两次
    pub 付费攻占: bool,
}

/// 攻占区域
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum Region {
    #[serde(rename = "东海")]
    #[default]
    东海,
    #[serde(rename = "南荒")]
    南荒,
    #[serde(rename = "西泽")]
    西泽,
    #[serde(rename = "北寒")]
    北寒,
}

impl Region {
    /// 转换为接口参数值
    pub(crate) fn api_value(&self) -> &str {
        match self {
            Self::东海 => "1",
            Self::南荒 => "2",
            Self::西泽 => "3",
            Self::北寒 => "4",
        }
    }
}

/// 帮派商会
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct BangPaiShangHui {
    /// 物品交易
    pub 交易会所: JiaoYiHuiSuo,
    /// 兑换物品
    pub 兑换商店: DuiHuanShangDian,
}

/// 交易会所
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JiaoYiHuiSuo {
    #[serde(rename = "孟婆汤*1")]
    pub 孟婆汤: bool,
    #[serde(rename = "还魂丹*20")]
    pub 还魂丹: bool,
    #[serde(rename = "悟性丹*20")]
    pub 悟性丹: bool,
    #[serde(rename = "百炼钢*20")]
    pub 百炼钢: bool,
    #[serde(rename = "大体力*5")]
    pub 大体力: bool,
    #[serde(rename = "挑战书*5")]
    pub 挑战书: bool,
    #[serde(rename = "经验药水*5")]
    pub 经验药水: bool,
    #[serde(rename = "黄金卷轴*5")]
    pub 黄金卷轴: bool,
    #[serde(rename = "神来拳套*5")]
    pub 神来拳套: bool,
    #[serde(rename = "还童卷轴*5")]
    pub 还童卷轴: bool,
    #[serde(rename = "神兵原石*5")]
    pub 神兵原石: bool,
    #[serde(rename = "软猥金丝*5")]
    pub 软猥金丝: bool,
    #[serde(rename = "凤凰羽毛*5")]
    pub 凤凰羽毛: bool,
    #[serde(rename = "潜能果实*5")]
    pub 潜能果实: bool,
    #[serde(rename = "奔流气息*5")]
    pub 奔流气息: bool,
    #[serde(rename = "上古玉髓*5")]
    pub 上古玉髓: bool,
    #[serde(rename = "经验木简*10")]
    pub 经验木简: bool,
    #[serde(rename = "大经验药水*1")]
    pub 大经验药水: bool,
    #[serde(rename = "投掷武器符文石*5")]
    pub 投掷武器符文石: bool,
    #[serde(rename = "小型武器符文石*5")]
    pub 小型武器符文石: bool,
    #[serde(rename = "中型武器符文石*5")]
    pub 中型武器符文石: bool,
    #[serde(rename = "大型武器符文石*5")]
    pub 大型武器符文石: bool,
    #[serde(rename = "巅峰之战二等勋章*2")]
    pub 巅峰之战二等勋章: bool,
}

impl JiaoYiHuiSuo {
    /// 返回所有交易物品的（匹配文本片段, 是否开启）列表
    ///
    /// 匹配时拼接中文逗号防止数量误匹配（如 *20 不会误匹配 *2）
    fn items(&self) -> [(&str, bool); 23] {
        [
            ("孟婆汤*1，", self.孟婆汤),
            ("还魂丹*20，", self.还魂丹),
            ("悟性丹*20，", self.悟性丹),
            ("百炼钢*20，", self.百炼钢),
            ("大体力*5，", self.大体力),
            ("挑战书*5，", self.挑战书),
            ("经验药水*5，", self.经验药水),
            ("黄金卷轴*5，", self.黄金卷轴),
            ("神来拳套*5，", self.神来拳套),
            ("还童卷轴*5，", self.还童卷轴),
            ("神兵原石*5，", self.神兵原石),
            ("软猥金丝*5，", self.软猥金丝),
            ("凤凰羽毛*5，", self.凤凰羽毛),
            ("潜能果实*5，", self.潜能果实),
            ("奔流气息*5，", self.奔流气息),
            ("上古玉髓*5，", self.上古玉髓),
            ("经验木简*10，", self.经验木简),
            ("大经验药水*1，", self.大经验药水),
            ("投掷武器符文石*5，", self.投掷武器符文石),
            ("小型武器符文石*5，", self.小型武器符文石),
            ("中型武器符文石*5，", self.中型武器符文石),
            ("大型武器符文石*5，", self.大型武器符文石),
            ("巅峰之战二等勋章*2，", self.巅峰之战二等勋章),
        ]
    }

    /// 判断 tips 是否匹配任一已开启的交易物品
    pub fn is_match(&self, tips: &str) -> bool {
        self.items()
            .iter()
            .any(|(s, enabled)| *enabled && tips.contains(s))
    }
}

/// 兑换商店
///
/// 物品名称列表，上限 10 个，不允许重复
#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct DuiHuanShangDian(Vec<String>);

impl Default for DuiHuanShangDian {
    fn default() -> Self {
        Self(vec!["泯灭·黑炎V碎片".to_string()])
    }
}

impl DuiHuanShangDian {
    /// 判断物品名是否在兑换列表中
    pub fn should_exchange(&self, goods_name: &str) -> bool {
        self.0.iter().any(|s| s == goods_name)
    }

    fn validate(&self) -> Result<()> {
        if self.0.len() > 10 {
            bail!("帮派商会.兑换商店 物品上限 10 个，当前 {} 个", self.0.len());
        }
        let unique: HashSet<_> = self.0.iter().collect();
        if unique.len() != self.0.len() {
            bail!("帮派商会.兑换商店 物品不允许重复");
        }
        Ok(())
    }
}

/// 侠士客栈
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct XiaShiKeZhan {
    pub 黑市商人: HeiShiShangRen,
}

impl XiaShiKeZhan {
    /// 根据奇遇 advId 判断是否开启交换
    pub fn is_enabled(&self, adv_id: &str) -> bool {
        self.黑市商人.is_enabled(adv_id)
    }
}

/// 黑市商人
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct HeiShiShangRen {
    #[serde(rename = "挑战书*1换取玄铁令*1")]
    pub ex_1006: bool,
    #[serde(rename = "黄金卷轴*2换取斗灵石空*5")]
    pub ex_1010: bool,
    #[serde(rename = "门派强化书*2换取斗灵石空*5")]
    pub ex_1011: bool,
    #[serde(rename = "斗灵石空*66换取V级万能碎片*1")]
    pub ex_1012: bool,
    #[serde(rename = "黄金卷轴*3换取斗神符*1")]
    pub ex_1013: bool,
    #[serde(rename = "无字天书*1换取易经八卦*1")]
    pub ex_1014: bool,
}

impl HeiShiShangRen {
    /// 根据奇遇 advId 判断是否开启交换
    pub fn is_enabled(&self, adv_id: &str) -> bool {
        match adv_id {
            "1006" => self.ex_1006,
            "1010" => self.ex_1010,
            "1011" => self.ex_1011,
            "1012" => self.ex_1012,
            "1013" => self.ex_1013,
            "1014" => self.ex_1014,
            _ => false,
        }
    }
}

/// 江湖长梦
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JiangHuChangMeng {
    pub 副本: ChangMengCopy,
    pub 兑换上限: ChangMengExchange,
}

impl JiangHuChangMeng {
    /// 根据 copy_name 返回执行次数上限，未配置的副本返回 0
    pub fn limit(&self, copy_name: &str) -> u32 {
        self.副本.limit(copy_name)
    }
}

/// 长梦副本
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ChangMengCopy {
    /// 执行次数上限，0~200
    pub 柒承的忙碌日常: u32,
}

impl ChangMengCopy {
    /// 根据 copy_name 返回执行次数上限，未配置的副本返回 0
    pub fn limit(&self, copy_name: &str) -> u32 {
        match copy_name {
            "柒承的忙碌日常" => self.柒承的忙碌日常,
            _ => 0,
        }
    }

    fn validate(&self) -> Result<()> {
        validate_range!("江湖长梦.副本.柒承的忙碌日常", self.柒承的忙碌日常, 0, 200);
        Ok(())
    }
}

/// 长梦兑换
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ChangMengExchange {
    /// 兑换上限，0~50
    pub 玄铁令: u32,
    /// 兑换上限，0~50
    pub 淬火结晶: u32,
    /// 兑换上限，0~50
    pub 石中剑: u32,
    /// 兑换上限，0~50
    pub 大型武器符咒: u32,
    /// 兑换上限，0~50
    pub 中型武器符咒: u32,
    /// 兑换上限，0~50
    pub 小型武器符咒: u32,
    /// 兑换上限，0~50
    pub 投掷武器符咒: u32,
}

impl ChangMengExchange {
    fn validate(&self) -> Result<()> {
        validate_range!("江湖长梦.兑换上限.玄铁令", self.玄铁令, 0, 50);
        validate_range!("江湖长梦.兑换上限.淬火结晶", self.淬火结晶, 0, 50);
        validate_range!("江湖长梦.兑换上限.石中剑", self.石中剑, 0, 50);
        validate_range!("江湖长梦.兑换上限.大型武器符咒", self.大型武器符咒, 0, 50);
        validate_range!("江湖长梦.兑换上限.中型武器符咒", self.中型武器符咒, 0, 50);
        validate_range!("江湖长梦.兑换上限.小型武器符咒", self.小型武器符咒, 0, 50);
        validate_range!("江湖长梦.兑换上限.投掷武器符咒", self.投掷武器符咒, 0, 50);
        Ok(())
    }
}

/// 深渊之潮
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ShenYuanZhiChao {
    /// 挑战副本
    pub 深渊秘境: ShenYuanMiJing,
}

/// 深渊秘境
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ShenYuanMiJing {
    /// 兑换副本
    pub 兑换: bool,
    /// 挑战副本
    pub 副本: ShenYuanMiJingCopy,
}

/// 深渊秘境副本
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone, Copy)]
pub enum ShenYuanMiJingCopy {
    #[serde(rename = "崎岖斗界")]
    #[default]
    崎岖斗界,
    #[serde(rename = "魂渡桥")]
    魂渡桥,
    #[serde(rename = "须臾之河")]
    须臾之河,
    #[serde(rename = "曲镜空洞")]
    曲镜空洞,
    #[serde(rename = "光影迷界")]
    光影迷界,
    #[serde(rename = "吞厄源头")]
    吞厄源头,
    #[serde(rename = "渊秘祭坛")]
    渊秘祭坛,
    #[serde(rename = "古帝遗迹")]
    古帝遗迹,
}

impl ShenYuanMiJingCopy {
    /// 转换为接口参数值
    pub fn api_id(&self) -> &str {
        match self {
            Self::崎岖斗界 => "1",
            Self::魂渡桥 => "2",
            Self::须臾之河 => "3",
            Self::曲镜空洞 => "4",
            Self::光影迷界 => "5",
            Self::吞厄源头 => "6",
            Self::渊秘祭坛 => "7",
            Self::古帝遗迹 => "8",
        }
    }
}

/// 龙凰之境
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LongHuangZhiJing {
    pub 兑换上限: LongHuangYunJi,
}

/// 龙凰云集
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LongHuangYunJi {
    /// 兑换上限，0~100
    pub 凰髓: u32,
    /// 兑换上限，0~16
    pub 凰火: u32,
    /// 兑换上限，0~100
    pub 龙玉: u32,
    /// 兑换上限，0~40
    pub 论武券: u32,
}

impl LongHuangYunJi {
    fn validate(&self) -> Result<()> {
        validate_range!("龙凰之境.兑换上限.凰髓", self.凰髓, 0, 100);
        validate_range!("龙凰之境.兑换上限.凰火", self.凰火, 0, 16);
        validate_range!("龙凰之境.兑换上限.龙玉", self.龙玉, 0, 100);
        validate_range!("龙凰之境.兑换上限.论武券", self.论武券, 0, 40);
        Ok(())
    }
}

/// 我的帮派
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WoDeBangPai {
    pub 报名: bool,
    pub 供奉: Vec<String>,
}

impl Default for WoDeBangPai {
    fn default() -> Self {
        Self {
            报名: false,
            供奉: vec!["还魂丹".to_string()],
        }
    }
}

impl WoDeBangPai {
    fn validate(&self) -> Result<()> {
        if self.供奉.len() > 40 {
            bail!("我的帮派.供奉 期望 0~40 个，当前 {} 个", self.供奉.len());
        }
        for (i, item) in self.供奉.iter().enumerate() {
            let chars = item.chars().count();
            if chars == 0 || chars > 15 {
                bail!(
                    "我的帮派.供奉[{}] 期望 1~15 字符，实际为 {} 字符：\"{item}\"",
                    i,
                    chars
                );
            }
        }
        Ok(())
    }
}

/// 门派邀请赛
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MenPaiYaoQingSai {
    pub 兑换: MenPaiExchange,
}

/// 门派邀请赛兑换
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MenPaiExchange {
    /// 兑换上限，0~20
    pub 炼气石: u32,
    /// 兑换上限，0~20
    pub 门派强化书: u32,
}

impl MenPaiExchange {
    fn validate(&self) -> Result<()> {
        validate_range!("门派邀请赛.兑换.炼气石", self.炼气石, 0, 20);
        validate_range!("门派邀请赛.兑换.门派强化书", self.门派强化书, 0, 20);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    // ─── GlobalConfig load 测试 ───

    fn config_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
        dir.path().join("global_config.json")
    }

    // 账号配置路径
    fn account_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
        dir.path().join("config.json")
    }

    // 写入 JSON 后加载全局配置
    fn load_global(json: &str) -> Result<GlobalConfig> {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        fs::write(&path, json).unwrap();
        GlobalConfig::load(&path)
    }

    // 写入 JSON 后加载账号配置
    fn load_account(json: &str) -> Result<AccountConfig> {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        fs::write(&path, json).unwrap();
        AccountConfig::load(&path)
    }

    // 默认值正确
    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert_eq!(config.运行时.并发数, 5);
        assert_eq!(config.运行时.日志保留天数, 30);
        assert_eq!(config.兑换码.code, "161616");
        assert_eq!(config.时空遗迹.八卦迷阵.第一层, BaGua::震);
        assert_eq!(config.时空遗迹.八卦迷阵.ids(), [7, 8, 2, 1]);
    }

    // 部分字段缺失时用默认值补齐
    #[test]
    fn test_load_partial_file_fills_defaults() {
        let config = load_global(r#"{}"#).unwrap();
        assert_eq!(config.运行时.并发数, 5);
        assert_eq!(config.运行时.日志保留天数, 30);
        assert_eq!(config.兑换码.code, "161616");
        assert_eq!(config.时空遗迹.八卦迷阵.第一层, BaGua::震);
        assert_eq!(config.时空遗迹.八卦迷阵.ids(), [7, 8, 2, 1]);
    }

    // 子对象存在但内部为空，用默认值补齐
    #[test]
    fn test_load_empty_sub_object_fills_defaults() {
        let config = load_global(r#"{"兑换码": {}}"#).unwrap();
        assert_eq!(config.兑换码.code, "161616");
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
        assert_eq!(config.兑换码.code, "161616");
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

    // ─── GlobalConfig update 测试 ───

    // 文件不存在时自动创建
    #[test]
    fn test_update_file_not_exists_creates_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        let diff = GlobalConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(content.contains("161616"));
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
        assert!(content.contains("161616"));
        assert!(!diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    // 废弃字段被移除，已有值保留
    #[test]
    fn test_update_removes_deprecated_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = config_path(&dir);
        // 包含一个不存在的旧字段
        let json = r#"{"兑换码": {"code": "161616"}, "废弃项": {"x": "y"}}"#;
        fs::write(&path, json).unwrap();
        let diff = GlobalConfig::update(&path).unwrap();
        let content = read_file(&path);
        assert!(!content.contains("废弃项"));
        assert!(content.contains("161616"));
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
        assert!(content.contains("161616"));
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

    // 运行时校验：边界值合法
    #[test]
    fn test_global_config_load_validate_range_ok() {
        let config = load_global(r#"{"运行时": {"并发数": 1, "日志保留天数": 90}}"#).unwrap();
        assert_eq!(config.运行时.并发数, 1);
        assert_eq!(config.运行时.日志保留天数, 90);
    }

    // 并发数超上限报错
    #[test]
    fn test_global_config_load_validate_concurrency_out_of_range() {
        assert!(load_global(r#"{"运行时": {"并发数": 21}}"#).is_err());
    }

    // 日志保留天数超上限报错
    #[test]
    fn test_global_config_load_validate_retention_out_of_range() {
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

    // ─── AccountConfig load 测试 ───

    // 默认值正确
    #[test]
    fn test_account_config_default() {
        let config = AccountConfig::default();
        assert_eq!(config.矿洞.开启副本.层数, KuangDongFloor::F1);
        assert_eq!(config.矿洞.开启副本.模式, KuangDongMode::Easy);
        assert!(!config.竞技场.兑换河图洛书);
        assert_eq!(config.梦想之旅.最多消耗梦幻机票数量, 3);
        assert!(!config.侠士客栈.黑市商人.ex_1006);
        assert_eq!(config.历练.乐斗顺序.len(), 9);
        assert_eq!(config.历练.乐斗顺序[0], LiLianBoss::XiongShi);
        assert_eq!(config.历练.乐斗顺序[8], LiLianBoss::XiangXian);
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

    // 梦想之旅机票数超上限报错
    #[test]
    fn test_account_config_load_validate_range() {
        assert!(load_account(r#"{"梦想之旅": {"最多消耗梦幻机票数量": 10}}"#).is_err());
    }

    // 梦想之旅机票数边界值合法
    #[test]
    fn test_account_config_load_validate_range_ok() {
        let config = load_account(r#"{"梦想之旅": {"最多消耗梦幻机票数量": 9}}"#).unwrap();
        assert_eq!(config.梦想之旅.最多消耗梦幻机票数量, 9);
    }

    // 掠夺目标战力超上限报错
    #[test]
    fn test_account_config_load_lue_duo_target_out_of_range() {
        assert!(load_account(r#"{"掠夺": {"目标战力": 100000}}"#).is_err());
    }

    // 掠夺战力增量超上限报错
    #[test]
    fn test_account_config_load_lue_duo_increment_out_of_range() {
        assert!(load_account(r#"{"掠夺": {"战力增量": 10000}}"#).is_err());
    }

    // 掠夺边界值合法
    #[test]
    fn test_account_config_load_lue_duo_range_ok() {
        let json = r#"{"掠夺": {"目标战力": 99999, "战力增量": 9999}}"#;
        let config = load_account(json).unwrap();
        assert_eq!(config.掠夺.目标战力, 99999);
        assert_eq!(config.掠夺.战力增量, 9999);
    }

    // 会武卷轴数量超上限报错
    #[test]
    fn test_account_config_load_hui_wu_out_of_range() {
        assert!(load_account(r#"{"会武": {"兑换真黄金卷轴数量": 101}}"#).is_err());
    }

    // 会武卷轴数量边界值合法
    #[test]
    fn test_account_config_load_hui_wu_range_ok() {
        let config = load_account(r#"{"会武": {"兑换真黄金卷轴数量": 100}}"#).unwrap();
        assert_eq!(config.会武.兑换真黄金卷轴数量, 100);
    }

    // 龙凰之境凰髓超上限报错
    #[test]
    fn test_account_config_load_long_huang_huang_sui_out_of_range() {
        assert!(load_account(r#"{"龙凰之境": {"兑换上限": {"凰髓": 101}}}"#).is_err());
    }

    // 龙凰之境凰火超上限报错
    #[test]
    fn test_account_config_load_long_huang_huang_huo_out_of_range() {
        assert!(load_account(r#"{"龙凰之境": {"兑换上限": {"凰火": 17}}}"#).is_err());
    }

    // 龙凰之境龙玉超上限报错
    #[test]
    fn test_account_config_load_long_huang_long_yu_out_of_range() {
        assert!(load_account(r#"{"龙凰之境": {"兑换上限": {"龙玉": 101}}}"#).is_err());
    }

    // 龙凰之境论武券超上限报错
    #[test]
    fn test_account_config_load_long_huang_lun_wu_quan_out_of_range() {
        assert!(load_account(r#"{"龙凰之境": {"兑换上限": {"论武券": 41}}}"#).is_err());
    }

    // 龙凰之境边界值合法
    #[test]
    fn test_account_config_load_long_huang_range_ok() {
        let json =
            r#"{"龙凰之境": {"兑换上限": {"凰髓": 100, "凰火": 16, "龙玉": 100, "论武券": 40}}}"#;
        let config = load_account(json).unwrap();
        assert_eq!(config.龙凰之境.兑换上限.凰髓, 100);
        assert_eq!(config.龙凰之境.兑换上限.凰火, 16);
        assert_eq!(config.龙凰之境.兑换上限.龙玉, 100);
        assert_eq!(config.龙凰之境.兑换上限.论武券, 40);
    }

    // 江湖长梦玄铁令超上限报错
    #[test]
    fn test_account_config_load_chang_meng_xuan_tie_ling_out_of_range() {
        assert!(load_account(r#"{"江湖长梦": {"兑换上限": {"玄铁令": 51}}}"#).is_err());
    }

    // 江湖长梦淬火结晶超上限报错
    #[test]
    fn test_account_config_load_chang_meng_cui_huo_jie_jing_out_of_range() {
        assert!(load_account(r#"{"江湖长梦": {"兑换上限": {"淬火结晶": 51}}}"#).is_err());
    }

    // 江湖长梦石中剑超上限报错
    #[test]
    fn test_account_config_load_chang_meng_shi_zhong_jian_out_of_range() {
        assert!(load_account(r#"{"江湖长梦": {"兑换上限": {"石中剑": 51}}}"#).is_err());
    }

    // 江湖长梦大型武器符咒超上限报错
    #[test]
    fn test_account_config_load_chang_meng_da_xing_wu_qi_fu_zhou_out_of_range() {
        assert!(load_account(r#"{"江湖长梦": {"兑换上限": {"大型武器符咒": 51}}}"#).is_err());
    }

    // 江湖长梦中型武器符咒超上限报错
    #[test]
    fn test_account_config_load_chang_meng_zhong_xing_wu_qi_fu_zhou_out_of_range() {
        assert!(load_account(r#"{"江湖长梦": {"兑换上限": {"中型武器符咒": 51}}}"#).is_err());
    }

    // 江湖长梦小型武器符咒超上限报错
    #[test]
    fn test_account_config_load_chang_meng_xiao_xing_wu_qi_fu_zhou_out_of_range() {
        assert!(load_account(r#"{"江湖长梦": {"兑换上限": {"小型武器符咒": 51}}}"#).is_err());
    }

    // 江湖长梦投掷武器符咒超上限报错
    #[test]
    fn test_account_config_load_chang_meng_tou_zhi_wu_qi_fu_zhou_out_of_range() {
        assert!(load_account(r#"{"江湖长梦": {"兑换上限": {"投掷武器符咒": 51}}}"#).is_err());
    }

    // 江湖长梦兑换上限边界值合法
    #[test]
    fn test_account_config_load_chang_meng_range_ok() {
        let json = r#"{"江湖长梦": {"兑换上限": {"玄铁令": 50, "淬火结晶": 50, "石中剑": 50, "大型武器符咒": 50, "中型武器符咒": 50, "小型武器符咒": 50, "投掷武器符咒": 50}}}"#;
        let config = load_account(json).unwrap();
        assert_eq!(config.江湖长梦.兑换上限.玄铁令, 50);
        assert_eq!(config.江湖长梦.兑换上限.淬火结晶, 50);
        assert_eq!(config.江湖长梦.兑换上限.石中剑, 50);
        assert_eq!(config.江湖长梦.兑换上限.大型武器符咒, 50);
        assert_eq!(config.江湖长梦.兑换上限.中型武器符咒, 50);
        assert_eq!(config.江湖长梦.兑换上限.小型武器符咒, 50);
        assert_eq!(config.江湖长梦.兑换上限.投掷武器符咒, 50);
    }

    // 副本名称映射到执行次数上限，未配置的名称返回 0
    #[test]
    fn test_chang_meng_copy_limit() {
        let copy = ChangMengCopy {
            柒承的忙碌日常: 3
        };
        assert_eq!(copy.limit("柒承的忙碌日常"), 3);
        assert_eq!(copy.limit("未知副本"), 0);
    }

    // 江湖长梦副本执行次数超上限报错
    #[test]
    fn test_account_config_load_chang_meng_copy_out_of_range() {
        assert!(load_account(r#"{"江湖长梦": {"副本": {"柒承的忙碌日常": 201}}}"#).is_err());
    }

    // 江湖长梦副本执行次数边界值合法
    #[test]
    fn test_account_config_load_chang_meng_copy_range_ok() {
        let json = r#"{"江湖长梦": {"副本": {"柒承的忙碌日常": 200}}}"#;
        let config = load_account(json).unwrap();
        assert_eq!(config.江湖长梦.副本.柒承的忙碌日常, 200);
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

    // 梦想之旅机票数超上限报错且不破坏原文件
    #[test]
    fn test_account_config_update_range_validation_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = account_path(&dir);
        let json = r#"{"梦想之旅": {"最多消耗梦幻机票数量": 50}}"#;
        fs::write(&path, json).unwrap();
        assert!(AccountConfig::update(&path).is_err());
        assert_eq!(read_file(&path), json);
    }

    // 历练 BOSS顺序 数量不足校验
    #[test]
    fn test_account_config_load_li_lian_too_few() {
        assert!(load_account(r#"{"历练": {"乐斗顺序": ["凶尸-令狐冲"]}}"#).is_err());
    }

    // 历练 BOSS顺序 数量过多校验
    #[test]
    fn test_account_config_load_li_lian_too_many() {
        let json = r#"{"历练": {"乐斗顺序": ["凶尸-令狐冲","虾兵头目-丁春秋","夜叉元帅-丘处机","霹雳头领-小龙女","宋姜-韦小宝","大鹏-扫地僧","马大王-鹤笔翁","嗜血鬼王-韦一笑","象仙-赵敏","象仙-赵敏"]}}"#;
        assert!(load_account(json).is_err());
    }

    // 历练 BOSS顺序 重复校验
    #[test]
    fn test_account_config_load_li_lian_duplicate() {
        let json = r#"{"历练": {"乐斗顺序": ["凶尸-令狐冲","虾兵头目-丁春秋","夜叉元帅-丘处机","霹雳头领-小龙女","宋姜-韦小宝","大鹏-扫地僧","马大王-鹤笔翁","嗜血鬼王-韦一笑","嗜血鬼王-韦一笑"]}}"#;
        assert!(load_account(json).is_err());
    }

    // 历练 BOSS顺序 合法自定义配置
    #[test]
    fn test_account_config_load_li_lian_custom() {
        let json = r#"{"历练": {"乐斗顺序": ["象仙-赵敏","凶尸-令狐冲","虾兵头目-丁春秋","夜叉元帅-丘处机","霹雳头领-小龙女","宋姜-韦小宝","大鹏-扫地僧","马大王-鹤笔翁","嗜血鬼王-韦一笑"]}}"#;
        let config = load_account(json).unwrap();
        assert_eq!(config.历练.乐斗顺序[0], LiLianBoss::XiangXian);
        assert_eq!(config.历练.乐斗顺序[1], LiLianBoss::XiongShi);
        assert_eq!(config.历练.乐斗顺序.len(), 9);
    }

    // 历练 BOSS顺序 无效枚举值应报错
    #[test]
    fn test_account_config_load_li_lian_invalid_boss() {
        let json = r#"{"历练": {"乐斗顺序": ["凶尸-令狐冲","虾兵头目-丁春秋","夜叉元帅-丘处机","霹雳头领-小龙女","宋姜-韦小宝","大鹏-扫地僧","马大王-鹤笔翁","嗜血鬼王-韦一笑","不存在-BOSS"]}}"#;
        assert!(load_account(json).is_err());
    }

    // ─── WoDeBangPai validate 测试 ───

    // 供奉数量超限
    #[test]
    fn test_wo_de_bang_pai_too_many_offerings() {
        let config = WoDeBangPai {
            报名: false,
            供奉: (0..41).map(|i| format!("物品{i}")).collect(),
        };
        assert!(config.validate().is_err());
    }

    // 供奉元素超长
    #[test]
    fn test_wo_de_bang_pai_item_too_long() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["a".repeat(16)],
        };
        assert!(config.validate().is_err());
    }

    // 供奉空字符串拒绝
    #[test]
    fn test_wo_de_bang_pai_empty_offering() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["".to_string()],
        };
        assert!(config.validate().is_err());
    }

    // 中文按字符数计：11 个汉字（33 字节）合法
    #[test]
    fn test_wo_de_bang_pai_chinese_chars_valid() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["供".repeat(11)],
        };
        assert!(config.validate().is_ok());
    }

    // 超过 15 个字符拒绝
    #[test]
    fn test_wo_de_bang_pai_too_many_chars() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["供".repeat(16)],
        };
        assert!(config.validate().is_err());
    }

    // 合法供奉
    #[test]
    fn test_wo_de_bang_pai_valid() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["还魂丹".to_string(), "经验药水".to_string()],
        };
        assert!(config.validate().is_ok());
    }

    // 供奉超限 — 走完整加载链路
    #[test]
    fn test_account_config_load_wo_de_bang_pai_too_many_offerings() {
        let items = (0..41)
            .map(|i| format!("\"物品{i}\""))
            .collect::<Vec<_>>()
            .join(",");
        let json = format!("{{\"我的帮派\": {{\"供奉\": [{items}]}}}}");
        assert!(load_account(&json).is_err());
    }

    // 门派邀请赛炼气石兑换上限超限报错
    #[test]
    fn test_account_config_load_men_pai_lian_qi_shi_out_of_range() {
        assert!(load_account(r#"{"门派邀请赛": {"兑换": {"炼气石": 21}}}"#).is_err());
    }

    // 门派邀请赛门派强化书兑换上限超限报错
    #[test]
    fn test_account_config_load_men_pai_men_pai_qiang_hua_shu_out_of_range() {
        assert!(load_account(r#"{"门派邀请赛": {"兑换": {"门派强化书": 21}}}"#).is_err());
    }

    // 门派邀请赛兑换上限边界值合法
    #[test]
    fn test_account_config_load_men_pai_range_ok() {
        let json = r#"{"门派邀请赛": {"兑换": {"炼气石": 20, "门派强化书": 20}}}"#;
        let config = load_account(json).unwrap();
        assert_eq!(config.门派邀请赛.兑换.炼气石, 20);
        assert_eq!(config.门派邀请赛.兑换.门派强化书, 20);
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

    #[test]
    fn test_chars_to_ids_too_short() {
        let bagua = BaGuaMiZhen::default();
        assert!(bagua.chars_to_ids("乾坤").is_none());
    }

    #[test]
    fn test_chars_to_ids_too_many() {
        let bagua = BaGuaMiZhen::default();
        // 有效卦象超过 4 个也返回 None
        assert!(bagua.chars_to_ids("乾坤坎离震").is_none());
    }

    #[test]
    fn test_chars_to_ids_invalid_chars_mixed() {
        let bagua = BaGuaMiZhen::default();
        // 包含非法字符，有效卦象不足 4 个
        assert!(bagua.chars_to_ids("ab乾坤").is_none());
    }

    #[test]
    fn test_chars_to_ids_empty() {
        let bagua = BaGuaMiZhen::default();
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

    // ─── XiaShiKeZhan::is_enabled 测试 ───

    #[test]
    fn test_is_enabled_known_adv_ids() {
        let kezhan = XiaShiKeZhan {
            黑市商人: HeiShiShangRen {
                ex_1006: true,
                ex_1010: false,
                ex_1011: false,
                ex_1012: true,
                ex_1013: false,
                ex_1014: true,
            },
        };
        assert!(kezhan.is_enabled("1006"));
        assert!(!kezhan.is_enabled("1010"));
        assert!(!kezhan.is_enabled("1011"));
        assert!(kezhan.is_enabled("1012"));
        assert!(!kezhan.is_enabled("1013"));
        assert!(kezhan.is_enabled("1014"));
    }

    #[test]
    fn test_is_enabled_unknown_adv_id() {
        let kezhan = XiaShiKeZhan::default();
        assert!(!kezhan.is_enabled("9999"));
        assert!(!kezhan.is_enabled(""));
        assert!(!kezhan.is_enabled("1007"));
    }

    // ─── JiaoYiHuiSuo::is_match 测试 ───

    /// 构造所有字段为指定值的 JiaoYiHuiSuo
    fn all_items(v: bool) -> JiaoYiHuiSuo {
        JiaoYiHuiSuo {
            孟婆汤: v,
            还魂丹: v,
            悟性丹: v,
            百炼钢: v,
            大体力: v,
            挑战书: v,
            经验药水: v,
            黄金卷轴: v,
            神来拳套: v,
            还童卷轴: v,
            神兵原石: v,
            软猥金丝: v,
            凤凰羽毛: v,
            潜能果实: v,
            奔流气息: v,
            上古玉髓: v,
            经验木简: v,
            大经验药水: v,
            投掷武器符文石: v,
            小型武器符文石: v,
            中型武器符文石: v,
            大型武器符文石: v,
            巅峰之战二等勋章: v,
        }
    }

    #[test]
    fn test_is_match_items_count() {
        let huisuo = JiaoYiHuiSuo::default();
        assert_eq!(huisuo.items().len(), 23);
    }

    #[test]
    fn test_is_match_all_enabled() {
        let huisuo = all_items(true);
        for (s, enabled) in huisuo.items() {
            assert!(enabled, "items() 中 {s:?} 应为 true");
            let tip = format!("教主想要{s}他将返回给你200商会银币");
            assert!(huisuo.is_match(&tip), "开启后应匹配：{s}");
        }
    }

    #[test]
    fn test_is_match_all_disabled() {
        let huisuo = JiaoYiHuiSuo::default();
        for (s, _) in huisuo.items() {
            let tip = format!("教主想要{s}他将返回给你200商会银币");
            assert!(!huisuo.is_match(&tip), "默认关闭不应匹配：{s}");
        }
    }

    #[test]
    fn test_is_match_quantity_mismatch() {
        let huisuo = all_items(true);
        // 数量不对时不应匹配：*20 的用 *2 测，*5 的用 *50 测
        assert!(!huisuo.is_match("教主想要还魂丹*2，他将返回给你200商会银币"));
        assert!(!huisuo.is_match("教主想要大体力*50，他将返回给你200商会银币"));
        assert!(!huisuo.is_match("教主想要经验木简*1，他将返回给你200商会银币"));
        assert!(!huisuo.is_match("教主想要大经验药水*10，他将返回给你200商会银币"));
        assert!(!huisuo.is_match("教主想要巅峰之战二等勋章*1，他将返回给你200商会银币"));
    }

    // ─── DuiHuanShangDian 测试 ───

    #[test]
    fn test_dui_huan_shang_dian_default() {
        let d = DuiHuanShangDian::default();
        assert_eq!(d.0, vec!["泯灭·黑炎V碎片"]);
    }

    #[test]
    fn test_dui_huan_shang_dian_too_many() {
        let d = DuiHuanShangDian((0..11).map(|i| format!("物品{i}")).collect());
        assert!(d.validate().is_err());
    }

    #[test]
    fn test_dui_huan_shang_dian_duplicate() {
        let d = DuiHuanShangDian(vec![
            "物品1".to_string(),
            "物品2".to_string(),
            "物品1".to_string(),
        ]);
        assert!(d.validate().is_err());
    }

    #[test]
    fn test_dui_huan_shang_dian_valid() {
        let d = DuiHuanShangDian(vec!["物品1".to_string(), "物品2".to_string()]);
        assert!(d.validate().is_ok());
    }

    #[test]
    fn test_should_exchange() {
        let d = DuiHuanShangDian(vec!["a".to_string(), "b".to_string()]);
        assert!(d.should_exchange("a"));
        assert!(!d.should_exchange("c"));
    }
}
