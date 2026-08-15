//! 邪神秘宝
//!
//! 高级秘宝和极品秘宝免费抽奖

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "邪神秘宝";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        advanced: IfFree,
        extreme: IfFree,
    }

    #[derive(Deserialize)]
    struct IfFree {
        #[serde(rename = "ifFree")]
        if_free: String,
    }

    let data: Query = match d.get("cmd=tenlottery&op=0").await {
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

    // 高级免费
    if data.advanced.if_free == "1" {
        免费抽奖(d, 0).await;
    }

    // 极品免费
    if data.extreme.if_free == "1" {
        免费抽奖(d, 1).await;
    }
}

async fn 免费抽奖(d: &DaLeDou, t: u8) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 免费抽奖
    let cmd = format!("cmd=tenlottery&op=2&type={t}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
