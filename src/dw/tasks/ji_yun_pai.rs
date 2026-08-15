//! 激运牌
//!
//! 领取、翻牌

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "激运牌";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default, rename = "cardNum")]
    card_num: String, // 激运牌数量
    #[serde(default)]
    mission: Vec<Mission>, // 任务列表
}

#[derive(Deserialize)]
struct Mission {
    id: String,
    status: String, // 是否可领取
}

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in &data.mission {
        if item.status == "1" {
            领取(d, &item.id).await;
        }
    }

    let Some(data) = query(d).await else {
        return;
    };

    let n: u64 = match data.card_num.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 card_num 字段失败：{e}"));
            return;
        }
    };

    我要翻牌(d, n).await;
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let data: Query = match d.get("cmd=realgoodsluck").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    // 不在活动时间
    if data.result == "-1" {
        return None;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

async fn 领取(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=realgoodsluck&op=getTaskReward&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 我要翻牌(d: &DaLeDou, n: u64) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
    }

    for _ in 0..n {
        // 我要翻牌
        let data: Response = match d.get("cmd=realgoodsluck&op=lotteryDraw").await {
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
    }
}
