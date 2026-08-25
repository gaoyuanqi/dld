//! 登录商店
//!
//! 兑换

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "登录商店";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize, Default)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        values: Values,
        #[serde(default)]
        items: Vec<Items>,
    }

    #[derive(Deserialize, Default)]
    struct Values {
        #[serde(default, rename = "8")]
        score: String,
    }

    #[derive(Deserialize)]
    struct Items {
        name: String,
        #[serde(default, rename = "type")]
        t: String,
    }

    let data: Query = match d.get("cmd=exchange&subtype=12").await {
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

    // 积分为0
    if data.values.score == "0" {
        return;
    }

    let want = d.config().登录商店.兑换.item_name();
    for item in &data.items {
        if item.name == want {
            兑换(d, &item.t, &data.values.score).await;
            return;
        }
    }

    d.log(TASK, &format!("未找到可兑换的 {want}"));
}

async fn 兑换(d: &DaLeDou, t: &str, times: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 兑换
    let cmd = format!("cmd=exchange&subtype=2&type={t}&times={times}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
