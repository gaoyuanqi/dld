//! 浩劫宝箱
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "浩劫宝箱";

// 浩劫宝箱有不同入口(查询subtype, 领取subtype)
const ACTIVITIES: &[(u32, u32)] = &[(142, 143), (151, 152)];

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default, rename = "giftFlag")]
        gift_flag: String, // 是否已领取
    }

    for &(q, t) in ACTIVITIES {
        let cmd = format!("cmd=newAct&subtype={q}");
        let data: Query = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                continue;
            }
        };

        // 不在活动时间
        if data.result == "-1" {
            continue;
        }

        if data.result != "0" {
            d.log(TASK, &data.msg);
            continue;
        }

        // 未领取
        if data.gift_flag == "0" {
            领取(d, t).await;
        }
    }
}

async fn 领取(d: &DaLeDou, t: u32) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=newAct&subtype={t}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
