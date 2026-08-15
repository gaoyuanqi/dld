//! 侠士客栈
//!
//! 领取奖励、奇遇、领取食盒
//!
//! 奇遇仅处理前来捣乱的xx、黑市商人（由账号配置决定是否交易）

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "侠士客栈";

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        draw: Vec<String>, // 领取状态
        #[serde(default, rename = "isOpen")]
        is_open: String, // 是否营业
    }

    // 一层查询
    let data: Query = match d.get("cmd=knight&op=13").await {
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

    // 一层已歇业
    if data.is_open == "0" {
        for (i, status) in data.draw.iter().enumerate() {
            // 未领取
            if status == "0" {
                let cmd = format!("cmd=knight&op=14&type=1&id={}", i + 1);
                领取奖励(d, &cmd).await;
            }
        }
    } else {
        领取二层奖励(d).await;
    }

    奇遇(d).await;
    共建回馈(d).await;
}

async fn 领取二层奖励(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        room: Vec<Room>, // 领取状态
        #[serde(default, rename = "isOpen")]
        is_open: String, // 是否营业
    }

    #[derive(Deserialize)]
    struct Room {
        draw: String, // 是否已领取
    }

    // 一层查询
    let data: Query = match d.get("cmd=knight&op=15").await {
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

    // 正在营业
    if data.is_open == "1" {
        return;
    }

    for (i, item) in data.room.iter().enumerate() {
        if item.draw == "0" {
            let cmd = format!("cmd=knight&op=14&type=2&id={}", i + 1);
            领取奖励(d, &cmd).await;
        }
    }
}

async fn 领取奖励(d: &DaLeDou, cmd: &str) {
    // 领取奖励
    let data: Response = match d.get(cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 奇遇(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        adventure: Vec<AdvenTure>, // 奇遇列表
    }

    #[derive(Deserialize)]
    struct AdvenTure {
        #[serde(rename = "advId")]
        adv_id: String, // 奇遇类型
        pos: String,
    }

    let data: Query = match d.get("cmd=knight&op=23").await {
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

    for item in &data.adventure {
        // 2xxx -> 前来捣乱的xx
        if item.adv_id.starts_with("2") || d.config().侠士客栈.is_enabled(&item.adv_id) {
            确认(d, &item.pos).await;
        }
    }
}

async fn 确认(d: &DaLeDou, pos: &str) {
    // 确认
    let cmd = format!("cmd=knight&op=25&pos={pos}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 共建回馈(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: i32,
        msg: String,
        #[serde(default)]
        build_list: Vec<BuildList>, // 酒盒列表
    }

    #[derive(Deserialize)]
    struct BuildList {
        status: u8, // 领取状态：0-可领取,1-已领取,2-未达门槛
        giftid: u32,
    }

    // 共建回馈
    let data: Query = match d.get("cmd=notice&op=view&sub=total").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != 0 {
        d.log(TASK, &data.msg);
        return;
    }

    for item in &data.build_list {
        // 可领取
        if item.status == 0 {
            领取(d, item.giftid).await;
        }
    }
}

async fn 领取(d: &DaLeDou, giftid: u32) {
    // 领取
    let cmd = format!("cmd=notice&op=reqreward&giftId={giftid}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
