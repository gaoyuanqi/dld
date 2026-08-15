//! 报名侠侣争霸

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "侠侣";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        apply: String,
    }

    let data: Query = match d.get("cmd=couplefight&subtype=1").await {
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

    // 未报名
    if data.apply == "0" {
        报名(d).await;
    }
}

async fn 报名(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 报名
    let data: Response = match d.get("cmd=couplefight&subtype=4").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
