//! 娃娃机
//!
//! 免费抓取一次

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "娃娃机";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default, rename = "isFree")]
        is_free: String, // 抓取是否免费
    }

    let data: Query = match d.get("cmd=newAct&subtype=114").await {
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

    if data.is_free == "1" {
        抓取一次(d).await;
    }
}

async fn 抓取一次(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 抓取一次
    let data: Response = match d.get("cmd=newAct&subtype=114&op=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
