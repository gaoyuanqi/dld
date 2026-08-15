//! 抢地盘
//!
//! 免费攻占、每日奖励
//!
//! 执行时间：每天 6:00–23:59

use chrono::{Local, Timelike};
use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "抢地盘";

pub async fn run(d: &DaLeDou) {
    let hour = Local::now().hour();
    if (0..5).contains(&hour) {
        return;
    }

    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
    }

    // 我的地盘
    let data: Query = match d.get("cmd=visitmanor").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 你不是领主
    if data.result == "-1" {
        if 免费(d).await {
            攻占(d).await;
        }
        return;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    每日奖励(d).await;
}

async fn 免费(d: &DaLeDou) -> bool {
    #[derive(Deserialize)]
    struct Record {
        result: String,
        msg: String,
        info: Vec<Info>,
    }

    #[derive(Deserialize)]
    struct Info {
        time: String,
    }

    // 抢地盘记录
    let data: Record = match d.get("cmd=showwulinmsg&type=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return false;
    }

    // 获取第一条记录
    let Some(item) = data.info.first() else {
        // 没记录说明有免费次数
        return true;
    };

    // 第一条记录：
    // 非当天格式为 MM/DD/HH/MM 返回true
    // 当天记录格式为 HH/MM 返回false
    item.time.len() != 5
}

async fn 攻占(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        manors: Vec<Manors>,
    }

    #[derive(Deserialize)]
    struct Manors {
        id: String,
    }

    // 无限制区
    let data: Query = match d.get("cmd=recommendmanor&type=11").await {
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

    // 第一个地盘
    let Some(item) = data.manors.first() else {
        return;
    };

    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
        repid: String,
    }

    // 攻占
    let cmd = format!("cmd=manorfight&type=1&manorid={}", item.id);
    let data: Response = match d.get(&cmd).await {
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

    抢地盘记录(d, &data.repid).await;
}

async fn 抢地盘记录(d: &DaLeDou, repid: &str) {
    #[derive(Deserialize)]
    struct Record {
        result: String,
        msg: String,
        info: Vec<Info>,
    }

    #[derive(Deserialize)]
    struct Info {
        desc: String,
        url: String,
    }

    // 抢地盘记录
    let data: Record = match d.get("cmd=showwulinmsg&type=1").await {
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

    for item in &data.info {
        if item.url == repid {
            d.log(TASK, &item.desc);
            return;
        }
    }
}

async fn 每日奖励(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 每日奖励
    let data: Response = match d.get("cmd=get&type=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
