//! 江湖长梦
//!
//! 副本优先金币，战败则结束回忆
//!
//! 周四兑换

use std::time::Duration;

use chrono::{Datelike, Local, Weekday};
use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

const TASK: &str = "江湖长梦";

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in &data.copy_list {
        // 跳过已过活动时间的副本
        if item.status == "0" {
            continue;
        }

        match item.name.as_str() {
            "柒承的忙碌日常" => 跑副本(d, item, 副本::柒承).await,
            "倚天屠龙归我心" => 跑副本(d, item, 副本::倚天).await,
            "绝世秘籍之争" => 跑副本(d, item, 副本::绝世).await,
            _ => continue,
        }
    }

    // 兑换商店只在周四开放
    if Local::now().weekday() != Weekday::Thu {
        return;
    }

    兑换商店(d).await;
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default, rename = "copyList")]
    copy_list: Vec<CopyList>, // 副本列表
}

#[derive(Deserialize)]
struct CopyList {
    id: String,
    name: String,     // 副本名称
    use_desc: String, // 香炉名称
    status: String,   // 副本状态，0=过期，1=空闲，2=进行中
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let data: Query = match d.get("cmd=jianghudream&op=showCopyInfo").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

/// 副本类型，对应事件处理逻辑
#[derive(Clone, Copy)]
enum 副本 {
    柒承,
    倚天,
    绝世,
}

/// 副本主流程：处理香炉、胜负、天数推进，事件选择交给 `kind` 对应的事件处理函数
async fn 跑副本(d: &DaLeDou, copy: &CopyList, kind: 副本) {
    let limit = d.config().江湖长梦.limit(&copy.name);
    if limit == 0 {
        return;
    }

    let Some(mut num) = get_item_num(d, &copy.use_desc).await else {
        return;
    };

    // 执行次数不超过配置上限
    num = num.min(limit);

    // 无香炉但副本进行中：至少跑一轮将其结束，避免卡住其他副本
    if num == 0 && copy.status == "2" {
        num = 1;
    }

    for _ in 0..num {
        let Some(mut data) = 开启副本(d, &copy.id).await else {
            return;
        };

        loop {
            // 战败
            if data.win == "1" {
                结束回忆(d, &copy.id).await;
                return;
            }

            let cur_days: u32 = match data.cur_days.parse() {
                Ok(v) => v,
                Err(e) => {
                    d.log(TASK, &format!("解析 {} cur_days 字段失败：{e}", copy.name));
                    return;
                }
            };

            let max_days: u32 = match data.max_days.parse() {
                Ok(v) => v,
                Err(e) => {
                    d.log(TASK, &format!("解析 {} max_days 字段失败：{e}", copy.name));
                    return;
                }
            };

            if cur_days > max_days {
                if !结束回忆(d, &copy.id).await {
                    return;
                }
                break;
            }

            if data.choose == "1" || cur_days == 0 {
                data = match 进入下一天(d).await {
                    Some(v) => v,
                    None => return,
                };
                continue;
            }

            let v = match kind {
                副本::柒承 => 柒承事件(d, &data, &copy.name).await,
                副本::倚天 => 倚天事件(d, &data, &copy.name, cur_days).await,
                副本::绝世 => 绝世事件(d, &data, &copy.name, cur_days).await,
            };
            if let Some(v) = v {
                data = v;
                continue;
            }

            if let Some(id) = 商店(&data) {
                let Some(v) = 选择事件(d, &copy.name, id).await else {
                    return;
                };
                data = v;
                continue;
            }

            return;
        }
    }
}

#[derive(Deserialize)]
struct BeiBao {
    result: String,
    msg: String,
    bag: Vec<Bag>,
}

#[derive(Deserialize)]
struct Bag {
    name: String,
    num: String,
}

async fn get_item_num(d: &DaLeDou, name: &str) -> Option<u32> {
    // 背包
    let cmd = format!("cmd=view&kind=0&sub=2&type=4&selfuin={}", d.qq());
    let data: BeiBao = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    let bag = data.bag.iter().find(|i| i.name == name)?;
    let num: u32 = match bag.num.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 {name} num 字段失败：{e}"));
            return None;
        }
    };

    Some(num)
}

/// 最多550金币
async fn 柒承事件(d: &DaLeDou, data: &Begin, copy_name: &str) -> Option<Begin> {
    if let Some(v) = 处理战斗(d, data, copy_name).await {
        return Some(v);
    }

    // 视而不见
    处理奇遇(d, data, copy_name, "2").await
}

/// 最多558金币
async fn 倚天事件(d: &DaLeDou, data: &Begin, copy_name: &str, cur_days: u32) -> Option<Begin> {
    if cur_days == 1 || cur_days == 7 {
        // 前辈、狠心离去
        return 处理奇遇(d, data, copy_name, "1").await;
    }

    if cur_days == 8 {
        // 独自神伤
        return 处理奇遇(d, data, copy_name, "3").await;
    }

    if let Some(v) = 处理战斗(d, data, copy_name).await {
        return Some(v);
    }

    // 开始回忆、回首掏
    处理奇遇(d, data, copy_name, "1").await
}

/// 最多490金币
async fn 绝世事件(d: &DaLeDou, data: &Begin, copy_name: &str, cur_days: u32) -> Option<Begin> {
    if cur_days == 1 || cur_days == 6 {
        // 携手合作、金银财宝
        return 处理奇遇(d, data, copy_name, "1").await;
    }

    // 只有奇遇事件，没有奇遇选项
    if cur_days == 3 {
        let id = 奇遇(data)?;
        return 选择事件(d, copy_name, id).await;
    }

    if cur_days == 4 {
        // 尝试交谈
        处理奇遇(d, data, copy_name, "2").await?;
        // 借机休息
        return 处理奇遇(d, data, copy_name, "2").await;
    }

    处理战斗(d, data, copy_name).await
}

async fn 处理战斗(d: &DaLeDou, data: &Begin, copy_name: &str) -> Option<Begin> {
    let id = 战斗(data)?;
    选择事件(d, copy_name, id).await
}

async fn 处理奇遇(
    d: &DaLeDou,
    data: &Begin,
    copy_name: &str,
    adventure_id: &str,
) -> Option<Begin> {
    let id = 奇遇(data)?;
    选择事件(d, copy_name, id).await?;
    奇遇选项(d, copy_name, adventure_id).await
}

/// 优先金币最多的战斗，无金币数字时回退首个任意战斗
fn 战斗(data: &Begin) -> Option<u8> {
    let mut best = None;
    let mut fallback = None;
    for (i, item) in data.event_list.iter().enumerate() {
        if item.t != "3" {
            continue;
        }
        let id = (i + 1) as u8;
        if fallback.is_none() {
            fallback = Some(id);
        }
        // 金币相同取先出现的
        if let Some(n) = 金币数(&item.info)
            && best.is_none_or(|(_, b)| n > b)
        {
            best = Some((id, n));
        }
    }

    best.map(|(id, _)| id).or(fallback)
}

/// 提取 info 中「金币：」后数字的最大值，无则返回 None
fn 金币数(info: &str) -> Option<u32> {
    let mut max: Option<u32> = None;
    for part in info.split("金币：").skip(1) {
        let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
        let Ok(n) = digits.parse::<u32>() else {
            continue;
        };
        max = Some(max.map_or(n, |m| m.max(n)));
    }

    max
}

fn 奇遇(data: &Begin) -> Option<u8> {
    for (i, item) in data.event_list.iter().enumerate() {
        let id = (i + 1) as u8;
        if item.t == "1" {
            return Some(id);
        }
    }

    None
}

fn 商店(data: &Begin) -> Option<u8> {
    for (i, item) in data.event_list.iter().enumerate() {
        let id = (i + 1) as u8;
        if item.t == "2" {
            return Some(id);
        }
    }

    None
}

#[derive(Deserialize)]
struct Begin {
    result: String,
    msg: String,
    win: String,    // 胜负
    choose: String, // 是否可以进入下一天
    #[serde(rename = "curDays")]
    cur_days: String, // 当前天数
    #[serde(rename = "maxDays")]
    max_days: String, // 最大天数
    #[serde(rename = "eventDesc")]
    event_desc: String, // 奇遇事件描述
    #[serde(rename = "eventList")]
    event_list: Vec<EventList>,
}

#[derive(Deserialize)]
struct EventList {
    #[serde(rename = "type")]
    t: String, // 事件类型
    #[serde(default)]
    info: String, // 只有战斗事件才有该字段
}

async fn 开启副本(d: &DaLeDou, id: &str) -> Option<Begin> {
    // 开启副本
    let cmd = format!("cmd=jianghudream&op=beginInstance&copyid={id}");
    let data: Begin = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

async fn 进入下一天(d: &DaLeDou) -> Option<Begin> {
    // 进入下一天
    let data: Begin = match d.get("cmd=jianghudream&op=goNextDay").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

/// 构造带副本名和天数前缀的事件日志
fn 事件日志(copy_name: &str, cur_days: &str, msg: &str) -> String {
    format!("{copy_name} => 第{cur_days}天：{msg}")
}

async fn 选择事件(d: &DaLeDou, copy_name: &str, event_id: u8) -> Option<Begin> {
    // 选择事件
    let cmd = format!("cmd=jianghudream&op=chooseEvent&event_id={event_id}");
    let data: Begin = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    let msg = if !data.msg.is_empty() {
        &data.msg
    } else {
        &data.event_desc
    };
    if !msg.is_empty() {
        d.log(TASK, &事件日志(copy_name, &data.cur_days, msg));
    }

    if data.result != "0" {
        return None;
    }

    Some(data)
}

async fn 奇遇选项(d: &DaLeDou, copy_name: &str, adventure_id: &str) -> Option<Begin> {
    // 奇遇选项
    let cmd = format!("cmd=jianghudream&op=chooseAdventure&adventure_id={adventure_id}");
    let data: Begin = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    d.log(TASK, &事件日志(copy_name, &data.cur_days, &data.event_desc));
    if data.result != "0" {
        return None;
    }

    Some(data)
}

async fn 结束回忆(d: &DaLeDou, id: &str) -> bool {
    // 结束回忆
    let data: Response = match d.get("cmd=jianghudream&op=endInstance").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    if data.result != "0" {
        return false;
    }

    领取首通奖励(d, id).await;

    true
}

async fn 领取首通奖励(d: &DaLeDou, id: &str) {
    // 领取首通奖励
    let cmd = format!("cmd=jianghudream&op=getFirstReward&copyid={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 兑换商店(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Exchange {
        result: String,
        msg: String,
        score: String,
        #[serde(rename = "goodsInfo")]
        goods_info: Vec<GoodsInfo>,
    }

    #[derive(Deserialize)]
    struct GoodsInfo {
        key_id: String,
        name: String,
        price: String,
        num: String, // 已兑换数量
        #[serde(rename = "maxNum")]
        max_num: String, // 最大兑换数量
    }

    // 兑换商店
    let data: Exchange = match d.get("cmd=longdreamexchange&op=viewIndex").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    let exchange = &d.config().江湖长梦.兑换上限;

    // 所有物品配置数量都为 0，无需兑换
    if exchange.玄铁令 == 0
        && exchange.淬火结晶 == 0
        && exchange.石中剑 == 0
        && exchange.大型武器符咒 == 0
        && exchange.中型武器符咒 == 0
        && exchange.小型武器符咒 == 0
        && exchange.投掷武器符咒 == 0
    {
        return;
    }

    let mut score: u32 = match data.score.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 score 字段失败：{e}"));
            return;
        }
    };

    // 积分门槛：低于最低价商品价格，无法兑换
    if score < 500 {
        return;
    }

    for goods in &data.goods_info {
        let want = match goods.name.as_str() {
            "玄铁令" => exchange.玄铁令,
            "淬火结晶" => exchange.淬火结晶,
            "石中剑" => exchange.石中剑,
            "大型武器符咒" => exchange.大型武器符咒,
            "中型武器符咒" => exchange.中型武器符咒,
            "小型武器符咒" => exchange.小型武器符咒,
            "投掷武器符咒" => exchange.投掷武器符咒,
            _ => continue,
        };
        if want == 0 {
            continue;
        }

        let num: u32 = match goods.num.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} num 字段失败：{e}", goods.name));
                continue;
            }
        };
        let max_num: u32 = match goods.max_num.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} max_num 字段失败：{e}", goods.name));
                continue;
            }
        };
        let price: u32 = match goods.price.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} price 字段失败：{e}", goods.name));
                continue;
            }
        };

        let max = calc_max_exchange(want, max_num, num, score, price);
        if max == 0 {
            continue;
        }
        score -= price * max;

        for _ in 0..max {
            兑换(d, &goods.key_id, &goods.name).await;
            time::sleep(Duration::from_millis(200)).await;
        }
    }
}

/// 计算本次可兑换数量
///
/// `want` 用户配置的兑换上限，`max_num` 服务器最大兑换数量，
/// `num` 已兑换数量，`score` 当前积分，`price` 单价
fn calc_max_exchange(want: u32, max_num: u32, num: u32, score: u32, price: u32) -> u32 {
    // 还需兑换 = min(配置上限, 服务器上限) - 已兑换，再受积分限制
    let need = want.min(max_num).saturating_sub(num);
    let affordable = score.checked_div(price).unwrap_or(0);
    need.min(affordable)
}

async fn 兑换(d: &DaLeDou, id: &str, name: &str) {
    // 兑换
    let cmd = format!("cmd=longdreamexchange&op=exchange&key_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("{name}*1 => {}", data.msg));
}

#[cfg(test)]
mod tests {
    use super::{Begin, EventList, calc_max_exchange, 战斗, 金币数};

    /// 构造事件：t 为类型，info 为描述
    fn event(t: &str, info: &str) -> EventList {
        EventList {
            t: t.to_string(),
            info: info.to_string(),
        }
    }

    /// 构造 Begin，仅 event_list 有意义
    fn begin(events: Vec<EventList>) -> Begin {
        Begin {
            result: String::new(),
            msg: String::new(),
            win: String::new(),
            choose: String::new(),
            cur_days: String::new(),
            max_days: String::new(),
            event_desc: String::new(),
            event_list: events,
        }
    }

    // 正常兑换：还需 40 个，积分可换 100 个，取 40
    #[test]
    fn test_calc_max_exchange_normal() {
        assert_eq!(calc_max_exchange(50, 50, 10, 10000, 100), 40);
    }

    // 配置上限高于服务器上限时取服务器上限
    #[test]
    fn test_calc_max_exchange_capped_by_server() {
        assert_eq!(calc_max_exchange(50, 20, 0, 10000, 100), 20);
    }

    // 已兑换达到配置上限，无需再兑换
    #[test]
    fn test_calc_max_exchange_already_exchanged() {
        assert_eq!(calc_max_exchange(30, 50, 30, 10000, 100), 0);
    }

    // 积分不足时受积分限制
    #[test]
    fn test_calc_max_exchange_limited_by_score() {
        assert_eq!(calc_max_exchange(50, 50, 0, 1000, 100), 10);
    }

    // 单价为 0 时不兑换（防除零）
    #[test]
    fn test_calc_max_exchange_zero_price() {
        assert_eq!(calc_max_exchange(50, 50, 0, 10000, 0), 0);
    }

    // 已兑换数超过服务器上限时按 0 处理
    #[test]
    fn test_calc_max_exchange_num_exceeds_max() {
        assert_eq!(calc_max_exchange(50, 50, 60, 10000, 100), 0);
    }

    // 金币多的战斗排在后面，优先返回金币最多的
    #[test]
    fn test_battle_prefers_most_coins() {
        let data = begin(vec![
            event("1", ""), // 奇遇
            event("3", "第1战：市井混混 金币：60\n等级：20"),
            event("3", "第1战：市井混混 金币：80\n等级：30"),
        ]);
        assert_eq!(战斗(&data), Some(3));
    }

    // 所有战斗 info 均无金币数字，回退首个任意战斗
    #[test]
    fn test_battle_falls_back_to_any() {
        let data = begin(vec![
            event("1", ""),
            event("3", "普通战斗"),
            event("3", "普通战斗二"),
        ]);
        assert_eq!(战斗(&data), Some(2));
    }

    // 无战斗事件返回 None
    #[test]
    fn test_battle_none() {
        let data = begin(vec![event("1", "金币"), event("2", "")]);
        assert_eq!(战斗(&data), None);
    }

    // 非战斗事件 info 含金币不算战斗
    #[test]
    fn test_battle_only_matches_battle_type() {
        let data = begin(vec![
            event("1", "第1战：市井混混 金币：99"),
            event("3", "第1战：市井混混 金币：60"),
        ]);
        assert_eq!(战斗(&data), Some(2));
    }

    // 同一 info 内多场战斗金币不同，取最大值
    #[test]
    fn test_battle_max_coins_within_info() {
        let data = begin(vec![event(
            "3",
            "第1战：市井混混 金币：60\n第2战：市井混混 金币：80",
        )]);
        assert_eq!(战斗(&data), Some(1));
    }

    // 提取单段金币数值
    #[test]
    fn test_coins_single() {
        assert_eq!(金币数("第1战：市井混混 金币：60\n等级：20"), Some(60));
    }

    // 多段金币取最大
    #[test]
    fn test_coins_max() {
        assert_eq!(金币数("第1战：金币：60\n第2战：金币：80"), Some(80));
    }

    // 无金币返回 None
    #[test]
    fn test_coins_none() {
        assert_eq!(金币数("普通战斗"), None);
    }

    // 金币后无数字返回 None
    #[test]
    fn test_coins_no_number() {
        assert_eq!(金币数("金币："), None);
    }
}
