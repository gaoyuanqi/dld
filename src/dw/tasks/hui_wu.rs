//! 会武
//!
//! 试炼、助威丐帮、领奖
//!
//! 高级试炼场战败会兑换一次试炼书
//!
//! 周四成功助威才兑换真黄金卷轴

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "会武";

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
        #[serde(default)]
        duration_type: String,
        #[serde(default)]
        scenes: Vec<Scenes>,
    }

    #[derive(Deserialize)]
    struct Scenes {
        id: String,
        status: String,
    }

    let data: Query = match d.get("cmd=sectmelee").await {
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

    // 试炼阶段
    if data.duration_type == "1" {
        for item in &data.scenes {
            // 已通关
            if item.status == "2" {
                continue;
            }

            if !试炼场(d, &item.id).await {
                return;
            }

            试炼(d, &item.id).await;
        }

        return;
    }

    // 助威时间
    if data.duration_type == "2" {
        冠军助威(d).await;
        return;
    }

    // 领奖时间
    if data.duration_type == "4" {
        领奖(d).await;
    }
}

async fn 试炼场(d: &DaLeDou, id: &str) -> bool {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
        #[serde(default)]
        dead: String, // 是否战败
    }

    // 试炼场
    let cmd = format!("cmd=sectmelee&op=showscene&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    // 非法参数，需通关前一个试炼场
    if data.result == "-1006" {
        return false;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return false;
    }

    data.dead == "0"
}

async fn 试炼(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
        #[serde(default)]
        dead: String, // 是否战败
        #[serde(default)]
        passed: String, // 是否通关
    }

    for _ in 0..11 {
        // 挑战
        let data: Response = match d.get("cmd=sectmelee&op=dotraining").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        if data.result != "0" {
            // 高级试炼场战败时兑换
            if data.result == "-1009" && id == "1002" && 兑换试炼书(d).await {
                continue;
            }
            return;
        }

        if data.dead == "1" && id != "1002" {
            return;
        }

        if data.passed == "1" && id != "1002" {
            return;
        }
    }
}

async fn 兑换试炼书(d: &DaLeDou) -> bool {
    // 兑换试炼书*1
    let cmd = "cmd=exchange&subtype=2&type=1265&times=1&costtype=13";
    let data: Response = match d.get(cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    data.result == "0"
}

async fn 冠军助威(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Showcheer {
        result: String,
        msg: String,
        cheer_sect: String,
    }

    // 冠军助威
    let data: Showcheer = match d.get("cmd=sectmelee&op=showcheer").await {
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

    // 还没有助威
    if data.cheer_sect == "0" {
        助威(d).await;
    }
}

async fn 助威(d: &DaLeDou) {
    // 助威丐帮
    let data: Response = match d.get("cmd=sectmelee&op=cheer&sect=1003").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
    if data.result != "0" {
        return;
    }

    兑换(d).await;
}

async fn 兑换(d: &DaLeDou) {
    let n = d.config().会武.兑换真黄金卷轴数量;
    if n == 0 {
        return;
    }

    // 兑换真黄金卷轴
    let cmd = format!("cmd=exchange&subtype=2&type=1263&times={n}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领奖(d: &DaLeDou) {
    // 领奖
    let data: Response = match d.get("cmd=sectmelee&op=drawreward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
