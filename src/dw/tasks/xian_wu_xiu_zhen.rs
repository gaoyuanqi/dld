//! 仙武修真
//!
//! 领取、寻访长留山

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "仙武修真";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default, rename = "immortalName")]
    immortal_name: String, // 仙人
    #[serde(default, rename = "leftNum")]
    left_num: String, // 剩余挑战次数
    #[serde(default)]
    task: Vec<Task>, // 任务列表
}

#[derive(Deserialize)]
struct Task {
    id: String,
    status: String,
}

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    // 寻仙
    let data: Query = match d.get("cmd=immortals&op=findimmortals").await {
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

    for item in &data.task {
        // 可领取
        if item.status == "1" {
            领取(d, &item.id).await;
        }
    }

    // 寻仙
    let data: Query = match d.get("cmd=immortals&op=findimmortals").await {
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

    if data.left_num == "0" {
        return;
    }

    let left_num: u8 = match data.left_num.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 left_num 字段失败：{e}"));
            return;
        }
    };

    for _ in 0..left_num {
        寻访(d).await;
        挑战(d).await;
    }
}

async fn 领取(d: &DaLeDou, id: &str) {
    // 领取
    let cmd = format!("cmd=immortals&op=getreward&taskid={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 寻访(d: &DaLeDou) {
    // 寻访长留山
    let data: Query = match d.get("cmd=immortals&op=visitimmortals&mountainId=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("{} => {}", data.immortal_name, data.msg));
}

async fn 挑战(d: &DaLeDou) {
    // 挑战
    let data: Response = match d.get("cmd=immortals&op=fightimmortals").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
