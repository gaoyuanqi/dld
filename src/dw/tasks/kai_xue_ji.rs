//! 开学季
//!
//! 领取疯狂许愿奖励

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "开学季";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    subtype: String,
    #[serde(default, rename = "type")]
    t: String,
    #[serde(default, rename = "subTypeInfo")]
    sub_type_info: Vec<SubTypeInfo>,
}

#[derive(Deserialize)]
struct SubTypeInfo {
    subtype: String,
    #[serde(default, rename = "subtypeName")]
    sub_type_name: String,
}

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    // {"result":"0","msg":"","timeDesc":"8月27日更新后至9月11日早6点","type":"0","subtype":"100","typeInfo":[{"type":"1","typeName":"神装"},{"type":"2","typeName":"铭刻"},{"type":"3","typeName":"专精"},{"type":"4","typeName":"强化"}],"subTypeInfo":[],"totalInfo":{}}
    let data: Query = match d.get("cmd=newAct&subtype=122").await {
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

    // 已领取具体奖励
    if data.subtype != "100" {
        return;
    }

    // 已选择种类
    if data.t != "0" {
        取消返回(d).await;
    }

    let want = d.config().开学季.选择.item_name();
    for t in 1..=4 {
        let Some(data) = 选择种类(d, t).await else {
            return;
        };

        for item in &data.sub_type_info {
            if item.sub_type_name != want {
                continue;
            }

            具体奖励(d, t, &item.subtype).await;
            return;
        }

        取消返回(d).await;
    }
}

async fn 取消返回(d: &DaLeDou) {
    // 取消返回
    let data: Response = match d.get("cmd=newAct&subtype=122&op=6").await {
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

async fn 选择种类(d: &DaLeDou, t: u8) -> Option<Query> {
    // 选择种类
    let cmd = format!("cmd=newAct&subtype=122&op=2&type={t}");
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

async fn 具体奖励(d: &DaLeDou, t: u8, subtype: &str) {
    // 具体奖励
    let cmd = format!("cmd=newAct&subtype=122&op=3&type={t}&sub_type={subtype}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
