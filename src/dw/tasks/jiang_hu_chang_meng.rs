//! 江湖长梦
//!
//! 副本优先战斗，战败则结束回忆
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
            "柒承的忙碌日常" => 柒承的忙碌日常(d, item).await,
            "倚天屠龙归我心" => 倚天屠龙归我心(d, item).await,
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

async fn 柒承的忙碌日常(d: &DaLeDou, copy: &CopyList) {
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

            if let Some(id) = 战斗(&data) {
                let Some(v) = 选择事件(d, id).await else {
                    return;
                };
                data = v;
                continue;
            }

            if let Some(id) = 奇遇(&data) {
                let Some(_) = 选择事件(d, id).await else {
                    return;
                };
                // 视而不见
                let Some(v) = 奇遇选项(d, "2").await else {
                    return;
                };
                data = v;
                continue;
            }

            if let Some(id) = 商店(&data) {
                let Some(v) = 选择事件(d, id).await else {
                    return;
                };
                data = v;
                continue;
            }

            return;
        }
    }
}

async fn 倚天屠龙归我心(d: &DaLeDou, copy: &CopyList) {
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

            if cur_days == 1 || cur_days == 7 {
                let Some(_) = 选择事件(d, 1).await else {
                    return;
                };
                // 前辈、狠心离去
                let Some(v) = 奇遇选项(d, "1").await else {
                    return;
                };
                data = v;
                continue;
            }

            if cur_days == 8 {
                let Some(_) = 选择事件(d, 1).await else {
                    return;
                };
                // 独自神伤
                let Some(v) = 奇遇选项(d, "3").await else {
                    return;
                };
                data = v;
                continue;
            }

            if let Some(id) = 战斗(&data) {
                let Some(v) = 选择事件(d, id).await else {
                    return;
                };
                data = v;
                continue;
            }

            if let Some(id) = 奇遇(&data) {
                let Some(_) = 选择事件(d, id).await else {
                    return;
                };
                // 开始回忆、回首掏
                let Some(v) = 奇遇选项(d, "1").await else {
                    return;
                };
                data = v;
                continue;
            }

            if let Some(id) = 商店(&data) {
                let Some(v) = 选择事件(d, id).await else {
                    return;
                };
                data = v;
                continue;
            }

            return;
        }
    }
}

fn 战斗(data: &Begin) -> Option<u8> {
    for (i, item) in data.event_list.iter().enumerate() {
        let id = (i + 1) as u8;
        if item.t == "3" {
            return Some(id);
        }
    }

    None
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

async fn 选择事件(d: &DaLeDou, event_id: u8) -> Option<Begin> {
    // 选择事件
    let cmd = format!("cmd=jianghudream&op=chooseEvent&event_id={event_id}");
    let data: Begin = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if !data.msg.is_empty() {
        d.log(TASK, &data.msg);
    } else if !data.event_desc.is_empty() {
        d.log(TASK, &data.event_desc);
    }

    if data.result != "0" {
        return None;
    }

    Some(data)
}

async fn 奇遇选项(d: &DaLeDou, adventure_id: &str) -> Option<Begin> {
    // 奇遇选项
    let cmd = format!("cmd=jianghudream&op=chooseAdventure&adventure_id={adventure_id}");
    let data: Begin = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    d.log(TASK, &data.event_desc);
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
    use super::calc_max_exchange;

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
}
