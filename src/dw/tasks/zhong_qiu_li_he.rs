//! 中秋礼盒
//!
//! 领取任务奖励

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "中秋礼盒";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        package_list: Vec<Package>,
    }

    #[derive(Deserialize)]
    struct Package {
        task_id: String,
        finish_num: String,
        limit_num: String,
    }

    let data: Query = match d.get("cmd=midautumngiftbag&sub=0").await {
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

    for item in &data.package_list {
        if item.finish_num == item.limit_num {
            领取(d, &item.task_id).await;
        }
    }
}

async fn 领取(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=midautumngiftbag&sub=1&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
