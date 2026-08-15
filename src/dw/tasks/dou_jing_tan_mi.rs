//! 斗境探秘
//!
//! 领取当天天和累计奖励

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "斗境探秘";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        currentday: String, // 当前天数
        #[serde(default)]
        task: Vec<Task>, // 累计奖励列表
        #[serde(default)]
        day: Vec<Day>, // 每日奖励列表
    }

    #[derive(Deserialize)]
    struct Task {
        id: String,     // 领取id
        value: String,  // 累计天数
        status: String, // 是否可领取
    }

    #[derive(Deserialize)]
    struct Day {
        id: String,     // 领取id
        day: String,    // 当前天数
        status: String, // 是否可领取
    }

    let data: Query = match d.get("cmd=newAct&subtype=166").await {
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

    // 领取当天奖励
    for item in &data.day {
        if item.day == data.currentday && item.status == "1" {
            领取(d, "2", &item.id).await;
            break;
        }
    }

    // 领取累计奖励
    for item in &data.task {
        if item.value == data.currentday && item.status == "1" {
            领取(d, "1", &item.id).await;
            break;
        }
    }
}

async fn 领取(d: &DaLeDou, op: &str, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=newAct&subtype=166&op=2&id={id}&&type={op}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
