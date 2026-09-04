//! 秘籍封印
//!
//! 领取秘籍和帮派礼包

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "秘籍封印";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        #[serde(default)]
        msg: String,
        #[serde(default, rename = "freeGift")]
        free_gift: Vec<Gift>, // 秘籍礼包
        #[serde(default, rename = "factionGift")]
        faction_gift: Vec<Gift>, // 帮派礼包
    }

    #[derive(Deserialize)]
    struct Gift {
        id: String,
        status: String,
    }

    let data: Query = match d.get("cmd=preferentialcheats").await {
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

    // 领取秘籍礼包
    for item in &data.free_gift {
        if item.status == "1" {
            领取(d, "5", &item.id).await;
        }
    }

    // 领取帮派礼包
    for item in &data.faction_gift {
        if item.status == "1" {
            领取(d, "4", &item.id).await;
        }
    }
}

async fn 领取(d: &DaLeDou, sub: &str, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=preferentialcheats&sub={sub}&gift={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
