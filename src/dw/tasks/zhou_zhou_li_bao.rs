//! 周周礼包
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "周周礼包";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        package_list: Vec<Package>,
    }

    #[derive(Deserialize)]
    struct Package {
        isawarded: String, // 是否可领取领取
        id: String,        // 领取id
    }

    let data: Query = match d.get("cmd=weekgiftbag&sub=0").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 不在活动时间返回系统繁忙
    if data.result == "-2" {
        return;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    for item in &data.package_list {
        // 可领取
        if item.isawarded == "0" {
            领取(d, &item.id).await;
            return;
        }
    }
}

async fn 领取(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=weekgiftbag&sub=1&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
