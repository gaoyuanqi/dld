//! 报名群侠

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "群侠";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        userapply: String,
    }

    let data: Query = match d.get("cmd=knightfight").await {
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
    if data.userapply == "0" {
        报名(d).await;
    }
}

async fn 报名(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 报名
    let data: Response = match d.get("cmd=knightfight&op=signup").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
