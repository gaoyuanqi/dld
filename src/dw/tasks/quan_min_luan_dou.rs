//! 全民乱斗
//!
//! 领取乱斗任务、任务列表（六门会武、武林盟主、武林大会）

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "全民乱斗";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    taskarray: Vec<TaskArray>, // 任务列表
}

#[derive(Deserialize)]
struct TaskArray {
    id: String,    // 领取id
    state: String, // 领取状态
}

pub async fn run(d: &DaLeDou) {
    let data: Query = match d.get("cmd=luandou").await {
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

    for acttype in 1..5 {
        任务列表(d, acttype).await;
    }
}

async fn 任务列表(d: &DaLeDou, acttype: u8) {
    // 任务列表
    let cmd = format!("cmd=luandou&op=9&acttype={acttype}");
    let data: Query = match d.get(&cmd).await {
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

    for item in &data.taskarray {
        if item.state == "3" {
            领取(d, &item.id).await;
        }
    }
}

async fn 领取(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=luandou&op=8&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
