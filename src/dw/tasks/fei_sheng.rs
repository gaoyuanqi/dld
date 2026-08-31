//! 飞升大作战
//!
//! 报名：优先报名单排（积分商城兑换玄铁令*1），失败则报名匹配
//! 领奖：领取排名和赛季奖励

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "飞升大作战";

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
        sign_status: String, // 是否已报名
    }

    let data: Query = match d.get("cmd=ascendheaven").await {
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
    if data.sign_status == "0" {
        if !报名单排(d).await {
            报名匹配(d).await;
        }

        领取排名奖励(d).await;
        领取赛季奖励(d).await;
    }
}

async fn 报名单排(d: &DaLeDou) -> bool {
    for _ in 0..2 {
        // 报名
        let data: Response = match d.get("cmd=ascendheaven&op=signup&type=1").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return false;
            }
        };

        d.log(TASK, &data.msg);
        if data.result == "0" {
            return true;
        }

        // {"result":"-1","msg":"当前为休赛期，无法报名排位比赛"}
        if data.result == "-1" {
            return false;
        }

        // 兑换玄铁令*1
        let data: Response = match d.get("cmd=ascendheaven&op=exchange&id=2&times=1").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return false;
            }
        };

        d.log(TASK, &data.msg);
        if !data.msg.contains("兑换玄铁令*1成功") {
            return false;
        }
    }

    true
}

async fn 报名匹配(d: &DaLeDou) {
    // 报名
    let data: Response = match d.get("cmd=ascendheaven&op=signup&type=2").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取排名奖励(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Viewra {
        result: String,
        msg: String,
        self_rank: String, // 排名
    }

    // 排位飞升榜
    let data: Viewra = match d.get("cmd=ascendheaven&op=viewrank").await {
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

    // 未上榜
    if data.self_rank == "0" {
        return;
    }

    // 领取排名奖励
    let data: Viewra = match d.get("cmd=ascendheaven&op=getrankgift").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取赛季奖励(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct ShowRealm {
        result: String,
        msg: String,
        history: Vec<History>,
    }

    #[derive(Deserialize)]
    struct History {
        season: String,
        status: String,
    }

    // 境界修为
    let data: ShowRealm = match d.get("cmd=ascendheaven&op=showrealm").await {
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

    for item in &data.history {
        if item.status != "1" {
            continue;
        }

        // 领取赛季奖励
        let cmd = format!("cmd=ascendheaven&op=getrealmgift&season={}", item.season);
        let res: Response = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &res.msg);
    }
}
