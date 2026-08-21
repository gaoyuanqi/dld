//! 猜单双
//!
//! 单双单双单

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "猜单双";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        status: String,
        #[serde(default)]
        list: Vec<serde_json::Value>,
    }

    let data: Query = match d.get("cmd=oddeven").await {
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

    if data.status == "0" && data.list.is_empty() {
        猜单双(d).await;
    }
}

async fn 猜单双(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
        status: String,
    }

    for v in ["1", "2", "1", "2", "1"] {
        // 猜单双
        let cmd = format!("cmd=oddeven&value={v}");
        let data: Response = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        if data.status == "1" {
            return;
        }
    }
}
