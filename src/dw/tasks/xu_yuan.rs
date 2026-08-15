//! 许愿
//!
//! 领取、领取许愿奖励

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "许愿";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    canchest: String, // 是否可领取连续许愿奖励
    fb: String,       // 许愿状态
}

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    if data.canchest == "1" {
        领取(d).await;
    }

    // 已许愿、未首胜
    if data.fb == "3" || data.fb == "4" {
        return;
    }

    if data.fb == "1" {
        领取许愿奖励(d).await;
    }

    let Some(data) = query(d).await else {
        return;
    };

    if data.fb == "2" {
        许愿(d).await;
    }
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let data: Query = match d.get("cmd=wish&sub=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

async fn 领取(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let data: Response = match d.get("cmd=wish&sub=6").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取许愿奖励(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        name: String,
        num: String,
    }

    // 领取许愿奖励
    let data: Response = match d.get("cmd=wish&sub=3").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("你获得了{}*{}", data.name, data.num));
}

async fn 许愿(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 许愿
    let data: Response = match d.get("cmd=wish&sub=2").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
