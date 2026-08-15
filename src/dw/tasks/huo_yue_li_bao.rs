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
        // #[serde(rename = "isGetLow")]
        // is_get_low: String, // 是否已领取50礼包
        #[serde(rename = "isGetHigh")]
        is_get_high: String, // 是否已领取80礼包
    }

    // 领取50礼包
    let data: Query = match d.get("cmd=newAct&subtype=85&op=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 不在活动时间
    if data.result == "-2" {
        return;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    // 已领过80礼包
    if data.is_get_high == "1" {
        return;
    }

    // 领取80礼包
    let data: Query = match d.get("cmd=newAct&subtype=85&op=2").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
