//! 乐斗黄历
//!
//! 领取、占卜

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "乐斗黄历";

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default, rename = "taskStatus")]
        task_status: String, // 是否可领取
    }

    let data: Query = match d.get("cmd=calender").await {
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

    if data.task_status == "1" {
        领取(d).await;
        运势占卜(d).await;
    }
}

async fn 领取(d: &DaLeDou) {
    // 领取
    let data: Response = match d.get("cmd=calender&op=gettortsh").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 运势占卜(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct ViewFate {
        result: String,
        msg: String,
        num: String,  // 龟甲数量
        fate: String, // 今日运势
    }

    // 查询占卜状态
    let data: ViewFate = match d.get("cmd=calender&op=viewfate").await {
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

    // 没有龟甲或者已经占卜过
    if data.num == "0" || !data.fate.is_empty() {
        return;
    }

    占卜(d).await;
}

async fn 占卜(d: &DaLeDou) {
    // 占卜
    let data: Response = match d.get("cmd=calender&op=dodivination").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
