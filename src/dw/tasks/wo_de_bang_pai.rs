//! 我的帮派
//!
//! 任务、供奉5次、帮战
//!
//! 周日领取奖励、报名帮战、激活祝福

use chrono::{Datelike, Local};

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "我的帮派";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(rename = "type")]
        t: String,
    }

    let data: Query = match d.get("cmd=viewfaction&id=0").await {
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

    // 没有加入帮派
    if data.t == "2" {
        return;
    }

    // 星期天
    if Local::now().weekday() == chrono::Weekday::Sun {
        帮战(d).await;
    }

    帮派任务(d).await;
}

async fn 帮派任务(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Task {
        result: String,
        msg: String,
        array: Vec<Array>,
    }

    #[derive(Deserialize)]
    struct Array {
        id: String,
        name: String,
        state: String,
    }

    // 任务
    let data: Task = match d.get("cmd=factiontask&sub=1").await {
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

    for item in &data.array {
        if item.state != "0" {
            continue;
        }

        match item.name.as_str() {
            "查看祭坛" => 查看(d, "cmd=altar").await,
            "粮草掠夺" => 查看(d, "cmd=forage_war").await,
            "查看踢馆" => 查看(d, "cmd=fac_challenge").await,
            "查看帮战" => 查看(d, "cmd=facwarrsp&id=1").await,
            "查看要闻" => 查看(d, "cmd=factioninfo&page=1").await,
            "查看帮贡" => 查看(d, "cmd=factiontask&sub=3").await,
            "帮战冠军" => 查看(d, "cmd=facwarrsp&id=1").await,
            "加速贡献" => 使用贡献药水(d).await,
            _ => {}
        }
    }

    // 任务
    let data: Task = match d.get("cmd=factiontask&sub=1").await {
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

    for item in &data.array {
        if item.state == "1" {
            领取任务奖励(d, &item.id).await;
        }
    }

    // 任一非 “帮派供奉” 任务可领取
    let has_other_task_completed = data
        .array
        .iter()
        .any(|i| i.state == "1" && i.name != "帮派供奉");
    if !has_other_task_completed {
        return;
    }

    帮派供奉(d).await;
}

async fn 查看(d: &DaLeDou, cmd: &str) {
    let data: Response = match d.get(cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
    }
}

async fn 使用贡献药水(d: &DaLeDou) {
    // 使用
    let cmd = format!("cmd=use&id=3038&selfuin={}", d.qq());
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取任务奖励(d: &DaLeDou, id: &str) {
    // 领取奖励
    let cmd = format!("cmd=factiontask&sub=2&taskid={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 帮派供奉(d: &DaLeDou) {
    let Some(data) = query_bag(d).await else {
        return;
    };

    let config_item = &d.config().我的帮派.供奉;
    for name in config_item {
        let Some(item) = data.bag.iter().find(|b| &b.name == name) else {
            continue;
        };

        if item.price == "0" {
            d.log(TASK, &format!("{name} => 非卖品不能用于供奉守护神"));
            continue;
        }

        let num: u32 = match item.num.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 num 字段失败：{e}"));
                return;
            }
        };

        if !供奉(d, &item.id, name, num).await {
            return;
        }
    }
}

async fn 供奉(d: &DaLeDou, id: &str, name: &str, num: u32) -> bool {
    let cmd = format!("cmd=feeddemo&id={id}");
    for _ in 0..num {
        let data: Response = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return false;
            }
        };

        d.log(TASK, &format!("{} => {}", name, data.msg));
        if data.result != "0" {
            return false;
        }

        if data.msg.starts_with("供奉成功，但守护神已经达到饥饿度上限") {
            return false;
        }
    }

    true
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    bag: Vec<Bag>,
}

#[derive(Deserialize)]
struct Bag {
    id: String,
    name: String,
    num: String,
    price: String,
}

async fn query_bag(d: &DaLeDou) -> Option<Query> {
    // 背包
    let cmd = format!("cmd=view&kind=0&sub=2&type=4&selfuin={}", d.qq());
    let data: Query = match d.get(&cmd).await {
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

async fn 帮战(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct FacWarrsp {
        result: String,
        msg: String,
        join: String, // 是否已报名帮战
    }

    // 一级帮八强
    let data: FacWarrsp = match d.get("cmd=facwarrsp&id=1").await {
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

    领取帮战奖励(d).await;

    // 未报名
    if data.join == "0" && d.config().我的帮派.报名 {
        报名(d).await;
        激活祝福(d).await;
        return;
    }

    // 已报名
    if data.join == "1" {
        激活祝福(d).await;
    }
}

async fn 报名(d: &DaLeDou) {
    // 报名
    let data: Response = match d.get("cmd=quicksighup").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取帮战奖励(d: &DaLeDou) {
    // 领取奖励
    let data: Response = match d.get("cmd=getwaraward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 激活祝福(d: &DaLeDou) {
    // 激活祝福
    let data: Response = match d.get("cmd=addbless").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
