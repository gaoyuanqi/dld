//! 挑战领奖
//!
//! 领取职业挑战排行奖励

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "挑战领奖";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default, rename = "myGetCond")]
        my_get_cond: String, // 是否可领取
    }

    let data: Query = match d.get("cmd=getawardbyrank&op=0").await {
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

    if data.my_get_cond == "1" {
        领取(d).await;
    }
}

async fn 领取(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let data: Response = match d.get("cmd=getawardbyrank&op=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
