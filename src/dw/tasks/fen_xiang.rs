//! 分享
//!
//! 每天分享、领取分享奖励、重置分享（全部奖励已领取时）
//!
//! 分享次数不足会挑战斗神塔（不会主动消耗付费次数）

use std::time::Duration;

use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "分享";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(rename = "dayShareTimes")]
    day_share_times: String, // 每日分享次数
    #[serde(rename = "daymaxsharetimes")]
    day_max_share_times: String, // 每日最大分享次数
    #[serde(rename = "shareTimes")]
    share_times: String, // 累计分享次数
    #[serde(rename = "totalTimes")]
    total_times: String, // 总计分享次数
    share_infos_: Vec<Info>,
}

#[derive(Deserialize)]
struct Info {
    #[serde(rename = "shareType")]
    share_type: String,
    #[serde(rename = "canShare")]
    can_share: String,
}

#[derive(Deserialize)]
struct TowerfightInfo {
    result: String,
    msg: String,
    status: String,          // 是否可继续下一层挑战
    day_left_times: String,  // 今日免费挑战次数
    left_fihgt_time: String, // 冷却时间
    cur_layer: String,       // 当前层数（还未挑战）
}

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    let Some(data) = share_query(d).await else {
        return;
    };

    // 已达分享次数上限
    if data.day_share_times == data.day_max_share_times {
        return;
    }

    for item in &data.share_infos_ {
        if item.can_share == "1" {
            分享(d, &item.share_type).await;
        }
    }

    斗神塔(d).await;

    领取奖励(d).await;
}

async fn share_query(d: &DaLeDou) -> Option<Query> {
    // 分享页面
    let data: Query = match d.get("cmd=shareinfo&subtype=3").await {
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

async fn 分享(d: &DaLeDou, info: &str) {
    // 分享
    let cmd = format!("cmd=shareinfo&subtype=1&shareinfo={}", info);
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 斗神塔(d: &DaLeDou) {
    let Some(data) = towerfight_info(d).await else {
        return;
    };

    // 已战败或者已通关
    if data.status == "2" || data.status == "3" {
        结束挑战(d).await;

        // 没有免费次数
        if data.day_left_times == "0/1" {
            return;
        }

        let Some(data) = towerfight_info(d).await else {
            return;
        };

        // 冷却时间
        let secs: u64 = data.left_fihgt_time.parse().unwrap_or(10);
        if secs > 0 {
            time::sleep(Duration::from_secs(secs)).await;
        }
    }

    // 没有免费次数且还没有开始挑战
    if data.day_left_times == "0/1" && data.status == "0" {
        return;
    }

    // 有免费次数
    // 没有免费次数但未战败

    for _ in 0..100 {
        挑战(d).await;

        let Some(data) = towerfight_info(d).await else {
            return;
        };

        // 冷却时间
        let secs: u64 = data.left_fihgt_time.parse().unwrap_or(10);
        if secs > 0 {
            time::sleep(Duration::from_secs(secs)).await;
        }

        // 已战败或者已通关
        if data.status == "2" || data.status == "3" {
            结束挑战(d).await;
            return;
        }

        let layer: u32 = match data.cur_layer.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 cur_layer 字段失败：{e}"));
                return;
            }
        };

        let challenged_layer = layer - 1;
        if !challenged_layer.is_multiple_of(10) {
            continue;
        }

        let Some(share_data) = share_query(d).await else {
            return;
        };

        if share_data.day_share_times != share_data.day_max_share_times {
            for item in &share_data.share_infos_ {
                if item.can_share == "1" {
                    分享(d, &item.share_type).await;
                }
            }

            continue;
        };

        d.log(
            TASK,
            &format!(
                "您今日的分享次数已达上限（{}/{}/{}）",
                share_data.day_share_times, share_data.share_times, share_data.total_times
            ),
        );

        自动挑战(d).await;

        let Some(data) = towerfight_info(d).await else {
            return;
        };

        // 可继续挑战，自动挑战没有通关
        if data.status == "1" {
            return;
        }

        // 冷却时间
        let secs: u64 = data.left_fihgt_time.parse().unwrap_or(10);
        if secs > 0 {
            time::sleep(Duration::from_secs(secs)).await;
        }

        结束挑战(d).await;
        return;
    }
}

async fn towerfight_info(d: &DaLeDou) -> Option<TowerfightInfo> {
    // 斗神塔页面
    let data: TowerfightInfo = match d.get("cmd=towerfight&type=3").await {
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

async fn 挑战(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Fight {
        result: String,
        msg: String,
        repid: String,
    }

    // 斗神塔挑战下一层
    let data: Fight = match d.get("cmd=towerfight&type=0").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &format!("斗神塔 =>{}", data.msg));
        return;
    }

    挑战记录(d, &data.repid).await;
}

async fn 挑战记录(d: &DaLeDou, repid: &str) {
    #[derive(Deserialize)]
    struct FightInfo {
        result: String,
        msg: String,
        info: Vec<RecordInfo>,
    }

    #[derive(Deserialize)]
    struct RecordInfo {
        url: String,
        desc: String,
    }

    // 斗神塔挑战记录
    let data: FightInfo = match d.get("cmd=towerfight&type=4").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &format!("斗神塔 =>{}", data.msg));
        return;
    }

    for item in &data.info {
        if item.url == repid {
            d.log(TASK, &format!("斗神塔 => {}", item.desc));
            return;
        }
    }
}

async fn 自动挑战(d: &DaLeDou) {
    // 斗神塔自动挑战
    let data: Response = match d.get("cmd=towerfight&type=10").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("斗神塔 => {}", data.msg));
}

async fn 结束挑战(d: &DaLeDou) {
    // 斗神塔结束挑战
    let data: Response = match d.get("cmd=towerfight&type=7").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("斗神塔 => {}", data.msg));
}

async fn 领取奖励(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct ShareAwardInfo {
        result: String,
        msg: String,
        share_done: String,
        share_infos_: Vec<AwardInfo>,
    }

    #[derive(Deserialize)]
    struct AwardInfo {
        #[serde(rename = "giftType")]
        gift_type: String,
        #[serde(rename = "getAward")]
        get_award: String,
    }

    let data: ShareAwardInfo = match d.get("cmd=shareinfo&subtype=4").await {
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

    if data.share_done == "0" {
        for item in &data.share_infos_ {
            if item.get_award == "1" {
                领奖(d, &item.gift_type).await;
                time::sleep(Duration::from_secs(2)).await;
            }
        }
    } else if data.share_done == "1" {
        重置分享(d).await;
    }
}

async fn 领奖(d: &DaLeDou, nums: &str) {
    // 领奖
    let cmd = format!("cmd=shareinfo&subtype=2&sharenums={}", nums);
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 重置分享(d: &DaLeDou) {
    // 重置分享
    let data: Response = match d.get("cmd=shareinfo&subtype=6").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
