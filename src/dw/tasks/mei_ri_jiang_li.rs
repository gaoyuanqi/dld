//! 每日奖励
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "每日奖励";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        login: LiBao,       // 每日礼包
        meridian: LiBao,    // 传功符礼包
        daren: LiBao,       // 达人礼包
        wuzitianshu: LiBao, // 无字天书礼包
    }

    #[derive(Deserialize)]
    struct LiBao {
        status: String,
        key: String,
    }

    let data: Query = match d.get("cmd=dailygift").await {
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

    if &data.login.status == "0" {
        领取(d, &data.login.key).await;
    }

    if &data.meridian.status == "0" {
        领取(d, &data.meridian.key).await;
    }

    if &data.daren.status == "0" {
        领取(d, &data.daren.key).await;
    }

    if &data.wuzitianshu.status == "0" {
        领取(d, &data.wuzitianshu.key).await;
    }
}

async fn 领取(d: &DaLeDou, key: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=dailygift&op=draw&key={key}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
