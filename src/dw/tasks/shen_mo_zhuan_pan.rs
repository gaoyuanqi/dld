//! 神魔转盘
//!
//! 免费幸运抽奖

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "神魔转盘";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default, rename = "freeTimes")]
        free_times: String, // 抽奖是否免费
    }

    let data: Query = match d.get("cmd=newAct&subtype=81").await {
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

    if data.free_times == "1" {
        幸运抽奖(d).await;
    }
}

async fn 幸运抽奖(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 抓取一次
    let data: Response = match d.get("cmd=newAct&subtype=81&op=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
