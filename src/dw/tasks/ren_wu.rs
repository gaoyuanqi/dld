//! 任务
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "任务";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        tasklist: Vec<TaskList>,
    }

    #[derive(Deserialize)]
    struct TaskList {
        id: String,
        status: String,
    }

    let data: Query = match d.get("cmd=task&sub=1").await {
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

    for item in &data.tasklist {
        // 可领取
        if item.status == "3" {
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
    let cmd = format!("cmd=task&sub=4&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
