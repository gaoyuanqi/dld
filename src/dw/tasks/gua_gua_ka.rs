//! 刮刮卡
//!
//! 领取、刮卡

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "刮刮卡";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        #[serde(default)]
        msg: String,
        #[serde(default)]
        active_info: Vec<ActiveInfo>,
        #[serde(default, rename = "cardNum")]
        card_num: String, // 刮刮卡数量
    }

    #[derive(Deserialize)]
    struct ActiveInfo {
        id: String,
        state: String,
        desc: String,
    }

    let data: Query = match d.get("cmd=newAct&subtype=139").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 不在活动时间
    if data.result == "-1" {
        return;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    for item in &data.active_info {
        if item.state == "1" {
            领取(d, &item.id, &item.desc).await;
        }
    }

    let data: Query = match d.get("cmd=newAct&subtype=139").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.card_num == "0" {
        return;
    }

    let card_num: u32 = match data.card_num.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 card_num 字段失败:{e}"));
            return;
        }
    };

    刮卡(d, card_num).await;
}

async fn 领取(d: &DaLeDou, id: &str, desc: &str) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        #[serde(default)]
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=newAct&subtype=139&op=2&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result == "0" {
        d.log(TASK, &format!("你获得{desc}"));
    } else {
        d.log(TASK, &data.msg);
    }
}

async fn 刮卡(d: &DaLeDou, card_num: u32) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
    }

    for _ in 0..card_num {
        // 刮卡
        let data: Response = match d.get("cmd=newAct&subtype=139&op=1").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        if data.result != "0" {
            return;
        }
    }
}
