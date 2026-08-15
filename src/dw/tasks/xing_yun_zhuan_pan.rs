//! 幸运转盘
//!
//! 转动转盘

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "幸运转盘";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        left_times: String, // 剩余转动次数
    }

    let data: Query = match d.get("cmd=newAct&subtype=50").await {
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

    if data.left_times == "0" {
        return;
    }

    转动转盘(d).await;
}

async fn 转动转盘(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 转动转盘
    let data: Response = match d.get("cmd=newAct&subtype=50&op=roll").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
