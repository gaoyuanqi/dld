//! 武林盟主
//!
//! 领取竞猜和排行奖励、报名黄金赛场、竞猜

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "武林盟主";

#[derive(Deserialize)]
struct Award {
    result: String,
    msg: String,
    #[serde(default)]
    award_info: Vec<AwardInfo>, // 奖励信息
}

#[derive(Deserialize)]
struct AwardInfo {
    #[serde(default)]
    section_id: String,
    #[serde(default)]
    round_id: String,
}

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
        status: String,
        #[serde(default)]
        signup_info: SignupInfo, // 报名信息
        #[serde(default)]
        guess_info: GuessInfo, // 竞猜信息
    }

    #[derive(Default, Deserialize)]
    struct SignupInfo {
        #[serde(default)]
        is_sign_up: String, // 报名状态
    }

    #[derive(Default, Deserialize)]
    struct GuessInfo {
        #[serde(default)]
        guess_confirm: String, // 竞猜状态
    }

    let data: Query = match d.get("cmd=wlmz&op=view_index").await {
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

    // 领奖时间
    if data.status == "1" {
        领取奖励(d).await;
    }

    let data: Query = match d.get("cmd=wlmz&op=view_index").await {
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

    // 报名时间并且还没有报名
    if data.status == "3" && data.signup_info.is_sign_up == "1" {
        参与报名(d).await;
        return;
    }

    // 竞猜时间并且未确认
    if data.status == "5" && data.guess_info.guess_confirm == "1" {
        前往竞猜(d).await;
    }
}

async fn 领取奖励(d: &DaLeDou) {
    let Some(data) = get_award_info(d).await else {
        return;
    };

    for item in &data.award_info {
        // 领取奖励
        let cmd = format!(
            "cmd=wlmz&op=get_award&section_id={}&round_id={}",
            item.section_id, item.round_id
        );
        let data: Response = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
    }
}

async fn get_award_info(d: &DaLeDou) -> Option<Award> {
    let data: Award = match d.get("cmd=wlmz&op=view_index").await {
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

async fn 参与报名(d: &DaLeDou) {
    // 报名黄金赛场
    let data: Response = match d.get("cmd=wlmz&op=signup&ground_id=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 前往竞猜(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Guess {
        result: String,
        msg: String,
        players: Vec<Players>, // 候选人
    }

    #[derive(Deserialize)]
    struct Players {
        index: String,
    }

    // 前往竞猜
    let data: Guess = match d.get("cmd=wlmz&op=view_guess").await {
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

    for item in &data.players {
        选择(d, &item.index).await;
    }

    确认竞猜选择(d).await;
}

async fn 选择(d: &DaLeDou, index: &str) {
    // 选择
    let cmd = format!("cmd=wlmz&op=guess_up&index={index}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 确认竞猜选择(d: &DaLeDou) {
    // 确认竞猜选择
    let data: Response = match d.get("cmd=wlmz&op=comfirm").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
