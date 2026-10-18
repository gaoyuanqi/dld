//! 重阳太白诗会
//!
//! 仅领取重阳礼包，不赋诗

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "重阳太白诗会";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        day_gift: String, // 是否可领重阳礼包
    }

    // {"show_poetry":"孤帆远影碧空尽的下一句","act_time":"10月16日6点后至10月23日6点","active":"10","day_gift":"0","poetry_gift":"0","upload_poetry":"0","result":"0","msg":""}
    let data: Query = match d.get("cmd=newAct&subtype=157&op=0").await {
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

    if data.day_gift == "1" {
        领取(d).await;
    }
}

async fn 领取(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let data: Response = match d.get("cmd=newAct&subtype=157&op=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
