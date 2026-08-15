//! 职业挑战
//!
//! 免费随机、挑战

use std::time::Duration;

use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "职业挑战";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        fresh_free_times: String, // 剩余免费随机次数
        #[serde(default)]
        fighted_times: String, // 挑战次数
    }

    let data: Query = match d.get("cmd=newAct&subtype=159").await {
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

    if data.fresh_free_times == "1" {
        随机(d).await;
    }

    if let Some(remaining) = data
        .fighted_times
        .split('/')
        .next()
        .and_then(|s| s.parse::<u8>().ok())
        && remaining > 0
    {
        挑战(d, remaining).await;
    }
}

async fn 随机(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 随机
    let data: Response = match d.get("cmd=newAct&subtype=159&op=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 挑战(d: &DaLeDou, n: u8) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
    }

    for _ in 0..n {
        // 挑战
        let data: Response = match d.get("cmd=newAct&subtype=159&op=5").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        if data.result != "0" {
            return;
        }
        time::sleep(Duration::from_millis(400)).await;
    }
}
