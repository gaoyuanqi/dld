//! 结拜
//!
//! 报名、助威（无限制区霸者之王）、领奖、领斗币

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "结拜";

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(rename = "groundId")]
        ground_id: String, // 报名分组id
        #[serde(rename = "hasSignFlag")]
        has_sign_flag: String, // 是否已报名
        #[serde(rename = "signTime")]
        sign_time: String, // 是否报名期
        #[serde(rename = "signInfo")]
        sign_info: String, // 报名信息
        #[serde(rename = "isSeaFighting")]
        is_sea_fighting: String, // 是否海选战斗中
        #[serde(rename = "isTopFighting")]
        is_top_fighting: String, // 是否决赛战斗中
    }

    let data: Query = match d.get("cmd=brofight&subtype=0").await {
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

    // 战斗期
    if data.is_sea_fighting == "1" || data.is_top_fighting == "1" {
        return;
    }

    // 报名期
    if data.sign_time == "1" {
        // 已报名
        if data.has_sign_flag == "1" {
            d.log(TASK, &data.sign_info);
            return;
        }
        报名(d, &data.ground_id).await;
        return;
    }

    // 非报名期：查战况视图决定助威还是领奖
    if data.sign_time == "0" {
        战况(d, &data.ground_id).await;
    }
}

async fn 报名(d: &DaLeDou, ground_id: &str) {
    // 报名
    let cmd = format!("cmd=brofight&subtype=6&ground_id={ground_id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 战况(d: &DaLeDou, ground_id: &str) {
    #[derive(Deserialize)]
    struct ZhanKuang {
        result: String,
        #[serde(rename = "cheerTime")]
        cheer_time: String, // 是否助威时间
        #[serde(rename = "hasCheer")]
        has_cheer: String, // 是否已助威
    }

    // 助威页面
    let cmd = format!("cmd=brofight&subtype=13&gid={ground_id}");
    let data: ZhanKuang = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        return;
    }

    // 助威时段未助威 → 助威
    if data.cheer_time == "1" && data.has_cheer == "0" {
        助威(d).await;
        return;
    }

    // 非助威时段 → 领奖、领斗币
    if data.cheer_time == "0" {
        领奖(d).await;
        斗币福利(d).await;
    }
}

async fn 助威(d: &DaLeDou) {
    // 助威霸者之王
    let data: Response = match d.get("cmd=brofight&subtype=2&team_id=6425586").await {
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
    let data: Response = match d.get("cmd=brofight&subtype=16").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 斗币福利(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Ground {
        result: String,
        msg: String,
        #[serde(rename = "groundList")]
        ground_list: Vec<Team>,
    }

    #[derive(Deserialize)]
    struct Team {
        uin: String,
        #[serde(rename = "remainMoney")]
        remain_money: String,
    }

    for gid in 1..=5 {
        // 领斗币页面
        let cmd = format!("cmd=brofight&subtype=0&gid={gid}");
        let data: Ground = match d.get(&cmd).await {
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

        for team in &data.ground_list {
            if team.remain_money != "0" {
                领斗币(d, &team.uin).await;
                return;
            }
        }
    }
}

async fn 领斗币(d: &DaLeDou, uin: &str) {
    // 领斗币
    let cmd = format!("cmd=brofight&subtype=1&player_uin={}", uin);
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
