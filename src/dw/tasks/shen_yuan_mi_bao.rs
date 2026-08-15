//! 深渊秘宝
//!
//! 三魂免费抽奖、七魄免费抽奖

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "深渊秘宝";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        soulflag: String, // 三魂抽奖是否免费
        #[serde(default)]
        mortalflag: String, // 七魄抽奖是否免费
    }

    let data: Query = match d.get("cmd=newAct&subtype=164").await {
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

    // 三魂免费抽奖
    if data.soulflag == "0" {
        领取(d, "1").await;
    }

    // 七魄免费抽奖
    if data.mortalflag == "0" {
        领取(d, "2").await;
    }
}

async fn 领取(d: &DaLeDou, t: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=newAct&subtype=164&op=1&type={t}&times=1");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
