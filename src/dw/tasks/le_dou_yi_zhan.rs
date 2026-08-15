//! 乐斗驿站
//!
//! 领取淬火结晶*1

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "乐斗驿站";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        if_got_award: String, // 是否已领取
    }

    let data: Query = match d.get("cmd=newAct&subtype=156").await {
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

    // 未领取
    if data.if_got_award == "0" {
        领取(d).await;
    }
}

async fn 领取(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let data: Response = match d.get("cmd=newAct&subtype=156&op=2").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
