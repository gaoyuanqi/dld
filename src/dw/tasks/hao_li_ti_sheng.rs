//! 好礼提升
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "好礼提升";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        times: String, // 领取状态
    }

    let data: Query = match d.get("cmd=newAct&subtype=37").await {
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

    // 可领取
    if data.times != "0" && !data.times.is_empty() {
        领取(d).await;
    }
}

async fn 领取(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let data: Response = match d.get("cmd=newAct&subtype=37&op=get").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
