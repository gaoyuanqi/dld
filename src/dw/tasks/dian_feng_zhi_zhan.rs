//! 巅峰之战
//!
//! 报名北派、领奖、征战
//!
//! 8级达人可以免除征战CD

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "巅峰之战";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        #[serde(default)]
        msg: String,
        status: String,
        win_group: String, // 获胜阵营
        userinfo: UserInfo,
    }

    #[derive(Deserialize)]
    struct UserInfo {
        group: String,  // 阵营
        relive: String, // 复活次数
        #[serde(default)]
        chall_status: String, // 挑战状态
    }

    let data: Query = match d.get("cmd=gvg&sub=0").await {
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

    // 报名期
    if data.status == "1" {
        // 未报名
        if data.userinfo.group == "0" {
            报名(d).await;
            领奖(d).await;
        }

        return;
    }

    // 已有阵营获胜
    if data.win_group != "0" {
        return;
    }

    // 非战斗期
    if data.status != "2" {
        return;
    }

    // 未报名
    if data.userinfo.group == "0" {
        return;
    }

    // 已死亡且无复活次数
    if data.userinfo.chall_status == "1" && data.userinfo.relive == "2" {
        return;
    }

    征战(d).await;
}

async fn 征战(d: &DaLeDou) {
    for _ in 0..14 {
        // 征战
        let data: Response = match d.get("cmd=gvg&sub=3").await {
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

async fn 报名(d: &DaLeDou) {
    // 报名北派
    let data: Response = match d.get("cmd=gvg&sub=1&group=2").await {
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
    let data: Response = match d.get("cmd=gvg&sub=4").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
