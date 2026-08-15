//! 徽章战令
//!
//! 领取每日礼包

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "徽章战令";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        ret: String,
        #[serde(default)]
        gift_status: String, // 是否已领取
    }

    let data: Query = match d.get("cmd=badge").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 不在活动时间
    if data.ret == "-1" {
        return;
    }

    if data.ret != "0" {
        d.log(TASK, &format!("ret={}", data.ret));
        return;
    }

    if data.gift_status == "0" {
        领取(d).await;
    }
}

async fn 领取(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let data: Response = match d.get("cmd=badge&op=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
