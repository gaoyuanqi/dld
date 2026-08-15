//! 器魂附魔
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "器魂附魔";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        missions: Vec<Mission>, // 任务列表
    }

    #[derive(Deserialize)]
    struct Mission {
        id: String,
        status: String,
    }

    let data: Query = match d.get("cmd=enchant&op=viewindex").await {
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

    for item in &data.missions {
        // 已领取或者未激活
        if item.status != "1" {
            continue;
        }
        领取(d, &item.id).await;
    }
}

async fn 领取(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=enchant&op=gettaskreward&missionId={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
