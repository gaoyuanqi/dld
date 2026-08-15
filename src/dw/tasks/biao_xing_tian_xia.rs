//! 镖行天下
//!
//! 领取奖励、护送、拦截
//!
//! 如果镖师是蔡八斗且有免费次数时则刷新镖师
//!
//! 拦截时跳过蔡八斗

use std::time::Duration;

use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "镖行天下";

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
        to_account: String, // 是否可领取奖励
        #[serde(default)]
        convey_count: String, // 已护送次数
        #[serde(default)]
        escort_state: String, // 护送状态
        #[serde(default)]
        looted_count: String, // 拦截次数
    }

    let data: Query = match d.get("cmd=cargo").await {
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

    // 押镖且护送完成
    if data.escort_state == "1" && data.to_account == "2" {
        领取奖励(d).await;
    }

    // 还没有护送
    if data.convey_count == "0" {
        刷新押镖(d).await;
        启程护送(d).await;
    }

    // 拦截已上限
    if data.looted_count == "3" {
        return;
    }

    拦截(d).await;
}

async fn 领取奖励(d: &DaLeDou) {
    // 领取奖励
    let data: Response = match d.get("cmd=cargo&op=16").await {
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

    走镖记录(d).await;
}

async fn 刷新押镖(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Cargo {
        result: String,
        msg: String,
        car_lvl: String,        // 镖师级别
        reselect_times: String, // 免费刷新次数
    }

    for _ in 0..2 {
        // 镖车界面
        let data: Cargo = match d.get("cmd=cargo&op=7").await {
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

        // 镖师不是蔡八斗（实习镖师）
        if data.car_lvl != "0" {
            return;
        }

        // 没有免费刷新次数
        if data.reselect_times == "0" {
            return;
        }

        // 刷新镖师
        let data: Response = match d.get("cmd=cargo&op=8").await {
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
    }
}

async fn 启程护送(d: &DaLeDou) {
    // 启程护送
    let data: Response = match d.get("cmd=cargo&op=6").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 拦截(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Cargo {
        result: String,
        msg: String,
        uin: String,
        looted_count: String, // 拦截次数
        passerbys: Vec<PasserBys>,
    }

    #[derive(Deserialize)]
    struct PasserBys {
        passerby_uin: String,
        escort_npc_name: String, // 镖师名称
    }

    for _ in 0..5 {
        // 刷新
        let data: Cargo = match d.get("cmd=cargo&op=3").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        if data.result == "-1" {
            time::sleep(Duration::from_secs(3)).await;
            continue;
        }
        if data.result != "0" {
            return;
        }

        for item in &data.passerbys {
            // 跳过蔡八斗、不能拦截自己镖车
            if item.escort_npc_name == "蔡八斗" || data.uin == item.passerby_uin {
                continue;
            }

            // 拦截
            let cmd = format!("cmd=cargo&op=14&passerby_uin={}", item.passerby_uin);
            let res: Cargo = match d.get(&cmd).await {
                Ok(v) => v,
                Err(e) => {
                    d.log(TASK, &format!("{e}"));
                    return;
                }
            };

            if res.result == "0" {
                走镖记录(d).await;

                // 拦截已上限
                if res.looted_count == "3" {
                    return;
                }

                time::sleep(Duration::from_millis(200)).await;
                continue;
            }

            d.log(TASK, &res.msg);
            time::sleep(Duration::from_millis(100)).await;
            if res.result == "-1" {
                continue;
            }
            if res.result != "0" {
                return;
            }
        }

        time::sleep(Duration::from_secs(3)).await;
    }
}

async fn 走镖记录(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Record {
        result: String,
        msg: String,
        feeds: Vec<Feeds>,
    }

    #[derive(Deserialize)]
    struct Feeds {
        desc: String,
    }

    // 走镖记录
    let data: Record = match d.get("cmd=cargo&op=4").await {
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

    // 第一条记录
    let Some(item) = data.feeds.first() else {
        return;
    };

    d.log(TASK, &item.desc);
}
