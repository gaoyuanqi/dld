//! 幸运金蛋
//!
//! 砸金蛋

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "幸运金蛋";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        am_status: String,
        #[serde(default)]
        pm_status: String,
    }

    let data: Query = match d.get("cmd=newAct&subtype=102").await {
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

    if data.am_status == "1" {
        砸金蛋(d, "0").await;
    } else if data.pm_status == "1" {
        砸金蛋(d, "1").await;
    }
}

async fn 砸金蛋(d: &DaLeDou, index: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 砸金蛋
    let cmd = format!("cmd=newAct&subtype=102&op=1&index={index}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
