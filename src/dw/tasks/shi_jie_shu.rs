//! 世界树
//!
//! 领取、免费温养

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "世界树";

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    灵宝树(d).await;
    源宝树(d).await;
}

async fn 灵宝树(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        gift: String, // 是否领取奖励
        #[serde(default)]
        login: String, // 是否可领取登录经验
        #[serde(default)]
        cost: String, // 是否可领取消费经验
    }

    // 灵宝树
    let data: Query = match d.get("cmd=worldtree&op=blesstree&type=1").await {
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

    // 可领取
    if data.login == "1" {
        领取经验(d, "cmd=worldtree&op=getexprience&id=1&taskid=1").await;
    }

    // 可领取
    if data.cost == "1" {
        领取经验(d, "cmd=worldtree&op=getexprience&id=1&taskid=2").await;
    }

    // 未领取
    if data.gift == "0" {
        领取奖励(d).await;
    }
}

async fn 领取经验(d: &DaLeDou, cmd: &str) {
    #[derive(Deserialize)]
    struct Exp {
        result: String,
        msg: String,
        lvname: String, // 等级名称
        exp: String,    // 当前经验
        limit: String,  // 下一级经验
    }

    // 领取经验
    let data: Exp = match d.get(cmd).await {
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

    d.log(
        TASK,
        &format!("领取经验 => {}（{}/{}）", data.lvname, data.exp, data.limit),
    );
}

async fn 领取奖励(d: &DaLeDou) {
    // 领取奖励
    let data: Response = match d.get("cmd=worldtree&op=getprice&type=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 源宝树(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default, rename = "freeStatus")]
        free_status: String, // 是否免费温养
        #[serde(default, rename = "weaponId")]
        weapon_id: String, // 温养武器id
    }

    // 源宝树
    let data: Query = match d.get("cmd=worldtree&op=viewexpandindex").await {
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

    // 已免费温养
    if data.free_status != "1" {
        return;
    }

    免费温养(d, &data.weapon_id).await;
}

async fn 免费温养(d: &DaLeDou, id: &str) {
    // 免费温养
    let cmd = format!("cmd=worldtree&op=dostrengh&weapon_id={id}&times=1");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
