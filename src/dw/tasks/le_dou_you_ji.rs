//! 乐斗游记
//!
//! 领取积分、一键领取、兑换

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "乐斗游记";

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
        maxscore: String, // 溢出积分
        #[serde(default)]
        lv: String, // 等级
        #[serde(default)]
        taskinfo: Vec<TaskInfo>, // 游记任务列表
    }

    #[derive(Deserialize)]
    struct TaskInfo {
        task_id: String,
        task_status: String,
    }

    let data: Query = match d.get("cmd=newAct&subtype=165").await {
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

    for item in &data.taskinfo {
        // 可领取
        if item.task_status == "1" {
            领取(d, &item.task_id).await;
            一键领取(d).await;
        }
    }

    // 未满级
    if data.lv != "60" {
        return;
    }

    let maxscore: u32 = match data.maxscore.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 maxscore 字段失败：{e}"));
            return;
        }
    };

    let (q, r) = (maxscore / 10, maxscore % 10);
    // 兑换十个
    for _ in 0..q {
        兑换(d, "10").await;
    }

    // 兑换一个
    for _ in 0..r {
        兑换(d, "1").await;
    }
}

async fn 领取(d: &DaLeDou, id: &str) {
    // 领取
    let cmd = format!("cmd=newAct&subtype=165&op=2&task_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 一键领取(d: &DaLeDou) {
    // 一键领取
    let data: Response = match d.get("cmd=newAct&subtype=165&op=6").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 兑换(d: &DaLeDou, n: &str) {
    // 领取
    let cmd = format!("cmd=newAct&subtype=165&op=3&num={n}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
