//! 豪侠出世
//!
//! 领取，包括签到好礼、侠士战令、豪侠战令

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "豪侠出世";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: i8,
        #[serde(default)]
        msg: String,
        #[serde(default)]
        free_list: Vec<List>, // 签到好礼
        #[serde(default)]
        low_list: Vec<List>, // 侠士战令
        #[serde(default)]
        high_list: Vec<List>, // 豪侠战令
    }

    #[derive(Deserialize)]
    struct List {
        giftid: u8,
        status: u8,
    }

    let data: Query = match d.get("cmd=knightdraw&op=view").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 不在活动时间
    if data.result == -1 {
        return;
    }

    if data.result != 0 {
        d.log(TASK, &data.msg);
        return;
    }

    for item in &data.free_list {
        if item.status == 0 {
            领取(d, item.giftid).await;
        }
    }

    for item in &data.low_list {
        if item.status == 0 {
            领取(d, item.giftid).await;
        }
    }

    for item in &data.high_list {
        if item.status == 0 {
            领取(d, item.giftid).await;
        }
    }
}

async fn 领取(d: &DaLeDou, id: u8) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=knightdraw&op=reqreward&sub=signin&ty=free&giftId={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
