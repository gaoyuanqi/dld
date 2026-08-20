//! 大侠回归
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "大侠回归";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        login: Vec<Task>, // 累计登录
        #[serde(default)]
        active: Vec<Task>, // 累计活跃度
        #[serde(default)]
        recharge: Vec<Task>, // 累计消耗
    }

    #[derive(Deserialize)]
    struct Task {
        id: String,
        #[serde(rename = "taskStatus")]
        task_status: String,
    }

    let data: Query = match d.get("cmd=newAct&subtype=162&op=1").await {
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

    // 领取累计登录
    for item in &data.login {
        if item.task_status == "1" {
            领取(d, &item.id).await;
        }
    }

    // 领取累计活跃度
    for item in &data.active {
        if item.task_status == "1" {
            领取(d, &item.id).await;
        }
    }

    // 领取累计消耗
    for item in &data.recharge {
        if item.task_status == "1" {
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
    let cmd = format!("cmd=newAct&subtype=162&op=2&taskid={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
