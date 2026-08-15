//! 元武登高
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "元武登高";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    // 领取
    let data: Response = match d.get("cmd=buyAct&subtype=1&op=getbonus&gift_id=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 不在活动时间、今日已领取
    if data.result == "-1" {
        return;
    }

    d.log(TASK, &data.msg);
}
