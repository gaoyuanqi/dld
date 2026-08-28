//! 时空遗迹
//!
//! 八卦迷阵：优先根据首通玩家名称提取，否则使用全局配置
//!
//! 异兽洞窟：如果全部通关则扫荡异兽母巢，否则按顺序挑战有血量的BOSS
//!
//! 联合征伐：挑战
//!
//! 悬赏任务：领取
//!
//! 赛季奖励：休赛期领取
//!
//! 遗迹商店：休赛期兑换，优先舆图，然后日月星特惠区、售卖区
//!
//! 休赛期第八周指上周四早 6 点到本周四早 6 点

use std::time::Duration;

use chrono::Utc;
use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "时空遗迹";

pub async fn run(d: &DaLeDou) {
    八卦迷阵(d).await;
    遗迹征伐(d).await;
}

async fn 八卦迷阵(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct BaGua {
        result: String,
        msg: String,
        status: String,
        power: String, // 耐力值
        first_uin: String,
    }

    // 八卦迷阵
    let data: BaGua = match d.get("cmd=spacerelic&op=goosipview").await {
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

    // 已通关领奖
    if data.status == "6" {
        return;
    }

    // 已通关未领奖
    if data.status == "5" {
        领取通关奖励(d).await;
        return;
    }

    // 卡在非第一层
    if data.status != "1" {
        return;
    }

    let power: u8 = match data.power.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 power 字段失败：{e}"));
            return;
        }
    };

    if power < 4 {
        return;
    }

    let ids = 八卦顺序(d, &data.first_uin);
    for id in &ids {
        if !选门(d, *id).await {
            return;
        }
    }

    领取通关奖励(d).await;
}

fn 八卦顺序(d: &DaLeDou, first_uin: &str) -> Vec<u8> {
    let config = &d.global_config().时空遗迹.八卦迷阵;

    // 优先从首通玩家名提取
    if let Some(seq) = first_uin.split('-').nth(1)
        && let Some(ids) = config.chars_to_ids(seq)
    {
        return ids;
    }

    // 兜底：用全局配置
    config.ids().to_vec()
}

#[derive(Deserialize)]
struct Response {
    msg: String,
}

async fn 领取通关奖励(d: &DaLeDou) {
    // 领取通关奖励
    let data: Response = match d.get("cmd=spacerelic&op=goosipaward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 选门(d: &DaLeDou, id: u8) -> bool {
    // 选门
    let cmd = format!("cmd=spacerelic&op=goosipview&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    data.msg.starts_with("恭喜您")
}

async fn 遗迹征伐(d: &DaLeDou) {
    // 休赛期
    if is_finish_time(d).await {
        悬赏任务(d).await;
        排名奖励(d).await;
        遗迹商店(d).await;
        return;
    }

    异兽洞窟(d).await;
    联合征伐(d).await;
    悬赏任务(d).await;
}

async fn is_finish_time(d: &DaLeDou) -> bool {
    #[derive(Deserialize)]
    struct FinishTime {
        result: String,
        msg: String,
        finishtime: String, // 赛季结束时间戳
    }

    let data: FinishTime = match d.get("cmd=spacerelic&op=finishtime").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return false;
    }

    let ts: i64 = match data.finishtime.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 finishtime 字段失败：{e}"));
            return false;
        }
    };

    let off_season = ts - 604_800;
    Utc::now().timestamp() >= off_season
}

async fn 排名奖励(d: &DaLeDou) {
    // 领取奖励
    let data: Response = match d.get("cmd=spacerelic&op=getrank").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 遗迹商店(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Shop {
        result: String,
        msg: String,
        score: String, // 总积分
        shopinfo1: Vec<Info>,
        shopinfo2: Vec<Info>,
    }

    #[derive(Deserialize)]
    struct Info {
        id: String,
        name: String,
        score: String,  // 消耗积分
        remain: String, // 剩余可兑换
    }

    // 遗迹商店
    let data: Shop = match d.get("cmd=spacerelic&op=shop").await {
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

    let mut score: u32 = match data.score.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 score 字段失败：{e}"));
            return;
        }
    };

    if score < 200 {
        return;
    }

    // 合并两个区的商品，按兑换优先级排序：舆图 → 日引石 → 月引石 → 星引石，同类型特惠区优先
    let mut items: Vec<&Info> = data.shopinfo1.iter().chain(data.shopinfo2.iter()).collect();
    let priority = ["4", "8", "1", "2", "3", "5", "6", "7"];
    items.sort_by_key(|item| {
        priority
            .iter()
            .position(|&p| p == item.id.as_str())
            .unwrap_or(usize::MAX)
    });

    for item in &items {
        if item.remain == "0" {
            continue;
        }

        let cost: u32 = match item.score.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} 单价失败：{e}", item.name));
                continue;
            }
        };

        if cost == 0 {
            d.log(TASK, &format!("{} 单价为：{cost}", item.name));
            continue;
        }

        if score < cost {
            continue;
        }

        let remain: u32 = match item.remain.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} 剩余数量失败：{e}", item.name));
                continue;
            }
        };

        let max = remain.min(score / cost);
        let (tens, ones) = (max / 10, max % 10);
        score -= cost * max;

        for _ in 0..tens {
            兑换(d, &item.name, &item.id, 10).await;
        }
        for _ in 0..ones {
            兑换(d, &item.name, &item.id, 1).await;
        }

        if score < 200 {
            return;
        }
    }
}

async fn 兑换(d: &DaLeDou, name: &str, id: &str, num: u8) {
    // 兑换
    let cmd = format!("cmd=spacerelic&op=buy&id={id}&num={num}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("{name}*{num} => {}", data.msg));
    time::sleep(Duration::from_millis(200)).await;
}

async fn 异兽洞窟(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Monster {
        result: String,
        msg: String,
        blood: String,
        num: String,
    }

    for id in 1..=5 {
        // 异兽洞窟
        let cmd = format!("cmd=spacerelic&op=monster&id={id}");
        let data: Monster = match d.get(&cmd).await {
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

        // 没有次数
        if data.num == "0" {
            return;
        }

        if data.blood != "0" {
            挑战(d, id).await;
            return;
        } else if id == 5 {
            扫荡(d, id).await;
            return;
        }
    }
}

async fn 挑战(d: &DaLeDou, id: u8) {
    // 挑战
    let cmd = format!("cmd=spacerelic&op=monsterfight&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 扫荡(d: &DaLeDou, id: u8) {
    // 扫荡
    let cmd = format!("cmd=spacerelic&op=saodang&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 联合征伐(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct ShowUnion {
        result: String,
        msg: String,
        selfinfo: Vec<SelfInfo>,
    }

    #[derive(Deserialize)]
    struct SelfInfo {
        dailyharm: String, // 当天伤害
    }

    // 联合征伐
    let data: ShowUnion = match d.get("cmd=spacerelic&op=showunion").await {
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

    let Some(selfinfo) = data.selfinfo.first() else {
        return;
    };

    if selfinfo.dailyharm == "0" {
        time::sleep(Duration::from_millis(800)).await;
        征伐挑战(d).await;
    }
}

async fn 征伐挑战(d: &DaLeDou) {
    // 挑战
    let data: Response = match d.get("cmd=spacerelic&op=bossfight").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 悬赏任务(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Task {
        result: String,
        msg: String,
        taskinfo1: Vec<Info>,
    }

    #[derive(Deserialize)]
    struct Info {
        finish: String,   // 任务进度
        giftflag: String, // 是否已领奖
        count: String,
        id: String,
        #[serde(rename = "type")]
        t: String, // 任务类型
    }

    // 悬赏任务
    let data: Task = match d.get("cmd=spacerelic&op=taskview").await {
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

    for item in &data.taskinfo1 {
        if item.finish == item.count && item.giftflag == "0" {
            领取(d, &item.t, &item.id).await;
        }
    }
}

async fn 领取(d: &DaLeDou, t: &str, id: &str) {
    // 领取
    let cmd = format!("cmd=spacerelic&op=finishtask&type={t}&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
