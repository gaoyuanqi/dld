//! 乐斗菜单
//!
//! 点单

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "乐斗菜单";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        #[serde(default)]
        msg: String,
        #[serde(default)]
        today: String,
        #[serde(default)]
        gift: String,
    }

    let data: Query = match d.get("cmd=menuact").await {
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

    if data.today == "1" {
        return;
    }

    let claimed: Vec<&str> = data.gift.split(',').filter(|s| !s.is_empty()).collect();
    for g in ["1", "2", "3", "4", "5"] {
        if !claimed.contains(&g) {
            点单(d, g).await;
            return;
        }
    }
}

async fn 点单(d: &DaLeDou, g: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 点单
    let cmd = format!("cmd=menuact&sub=1&gift={g}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
