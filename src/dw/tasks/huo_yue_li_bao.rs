//! 活跃礼包
//!
//! 领取50、80活跃礼包

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "活跃礼包";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(rename = "isGetLow")]
        is_get_low: String, // 是否已领取50礼包
        #[serde(rename = "isGetHigh")]
        is_get_high: String, // 是否已领取80礼包
    }

    // 活跃礼包
    let data: Query = match d.get("cmd=newAct&subtype=85&op=0").await {
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

    if data.is_get_low == "0" {
        领取(d, "1").await;
    }

    if data.is_get_high == "0" {
        领取(d, "2").await;
    }
}

async fn 领取(d: &DaLeDou, op: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=newAct&subtype=85&op={op}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // {"result":"-2","msg":"当前不在活动时间!","isGetLow":"0","isGetHigh":"0"}
    // {"result":"-2","msg":"您活跃度不足50点！","isGetLow":"0","isGetHigh":"0"}
    // {"result":"-2","msg":"您今天已抽取该奖励!","isGetLow":"1","isGetHigh":"0"}
    if !data.msg.starts_with("当") {
        d.log(TASK, &data.msg);
    }
}
