//! 登录有礼
//!
//! 领取登录礼包、领取充值礼包

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "登录有礼";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        free: Vec<u8>, // 登录礼包列表
        #[serde(default)]
        pay: Vec<u8>, // 充值礼包列表
    }

    let data: Query = match d.get("cmd=newAct&subtype=49").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 不在活动时间
    if data.result == "-102" {
        return;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    // 领取免费礼包（status: 1=可领取, 2=已领取, 3=未解锁）
    for (i, &status) in data.free.iter().enumerate() {
        if status == 1 {
            领取奖励(d, 1, i).await;
            break;
        }
    }

    // 领取充值礼包（status: 1=可领取, 2=已领取, 3=未解锁）
    for (i, &status) in data.pay.iter().enumerate() {
        if status == 1 {
            领取奖励(d, 2, i).await;
            break;
        }
    }
}

async fn 领取奖励(d: &DaLeDou, t: u8, i: usize) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    let cmd = format!("cmd=newAct&subtype=49&op=draw&gift_type={t}&gift_index={i}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
