//! 群雄逐鹿
//!
//! 报名、领奖

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "群雄逐鹿";

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        in_sign_up_time: String, // 是否在报名时间
        #[serde(default)]
        signed_up_zone: String, // 已报名赛区编号
    }

    let data: Query = match d.get("cmd=thronesbattle").await {
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

    // 已报名
    if data.signed_up_zone != "0" {
        领奖(d).await;
    } else if data.in_sign_up_time == "1" {
        报名(d).await;
    }
}

async fn 报名(d: &DaLeDou) {
    // 报名
    let data: Response = match d.get("cmd=thronesbattle&op=signup").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领奖(d: &DaLeDou) {
    // 领奖
    let data: Response = match d.get("cmd=thronesbattle&op=drawreward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
