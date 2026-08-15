//! 梦想之旅
//!
//! 普通旅行、周四梦幻旅行、周四领取
//!
//! 消耗梦幻机票条件：
//! 拥有梦幻机票数量 >= 最多消耗梦幻机票数量 >= 未去过

use chrono::{Datelike, Local};
use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "梦想之旅";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    normalticket: String, // 普通机票
    #[serde(default)]
    dreamticket: String, // 梦幻机票
    #[serde(default)]
    awardstatus: String, // 是否可领取终极奖励
    #[serde(default)]
    bmap_info: Vec<Info>,
    #[serde(default)]
    smap_info: Vec<Info>,
}

#[derive(Deserialize)]
struct Info {
    id: String,
    status: String,
}

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    // 普通旅行
    if data.normalticket == "1" {
        旅行(d, "0").await;
    }

    // 不是星期四
    if Local::now().weekday() != chrono::Weekday::Thu {
        return;
    }

    let Some(data) = query(d).await else {
        return;
    };

    let dreamy_count: u32 = match data.dreamticket.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 dreamticket 字段失败：{e}"));
            return;
        }
    };

    // 最多消耗梦幻机票数量
    let max_count: u32 = d.config().梦想之旅.最多消耗梦幻机票数量;

    // 统计未去过数量
    let unvisited: u32 = data.smap_info.iter().filter(|d| d.status == "0").count() as u32;

    if dreamy_count >= max_count && max_count >= unvisited {
        for item in &data.smap_info {
            // 未去过
            if item.status == "0" {
                旅行(d, &item.id).await;
            }
        }
    }

    let Some(data) = query(d).await else {
        return;
    };

    // 领取区域奖励
    for item in &data.bmap_info {
        // 可领取
        if item.status == "1" {
            领取(d, &item.id).await;
        }
    }

    let Some(data) = query(d).await else {
        return;
    };

    // 领取四个区域奖励后才能领取终极奖励
    if data.awardstatus == "1" {
        领取(d, "0").await;
    }
}

async fn query(d: &DaLeDou) -> Option<Query> {
    // 梦想之旅
    let data: Query = match d.get("cmd=dreamtrip").await {
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

async fn 旅行(d: &DaLeDou, id: &str) {
    // 普通旅行/梦幻旅行
    let cmd = format!("cmd=dreamtrip&sub=1&smapid={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取(d: &DaLeDou, id: &str) {
    // 领取区域/终极
    let cmd = format!("cmd=dreamtrip&sub=2&bmapid={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
