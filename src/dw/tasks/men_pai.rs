//! 门派
//!
//! 万年寺：点燃普通香炉和高香香炉
//!
//! 八叶堂：进入木桩训练和进入同门切磋
//!
//! 八叶堂：完成

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "门派";

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
    }

    let data: Query = match d.get("cmd=sect").await {
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

    万年寺(d).await;
    八叶堂(d).await;
    五花堂(d).await;
}

async fn 万年寺(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct ShowCouncil {
        result: String,
        msg: String,
        left_free_fumigate_times: String, // 剩余免费上香次数
        left_paid_fumigate_times: String, // 剩余付费上香次数
    }

    // 万年寺
    let data: ShowCouncil = match d.get("cmd=sect&op=showincense").await {
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

    if data.left_free_fumigate_times == "1" {
        普通香炉(d).await;
    }

    if data.left_paid_fumigate_times == "1" && d.config().门派.万年寺.付费高香香炉 {
        高香香炉(d).await;
    }
}

async fn 普通香炉(d: &DaLeDou) {
    // 点燃
    let data: Response = match d.get("cmd=sect&op=fumigate&type=free").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 高香香炉(d: &DaLeDou) {
    for _ in 0..2 {
        // 点燃
        let data: Response = match d.get("cmd=sect&op=fumigate&type=paid").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        if data.result == "0" {
            return;
        }

        if !兑换(d, "1248").await {
            return;
        }
    }
}

async fn 八叶堂(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct ShowTraining {
        result: String,
        msg: String,
        npc_challenged_times: String,    // NPC切磋次数
        member_challenged_times: String, // 同门切磋次数
    }

    // 八叶堂
    let data: ShowTraining = match d.get("cmd=sect&op=showtraining").await {
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

    if data.npc_challenged_times == "0" {
        木桩训练(d).await;
    }

    let is_paid = d.config().门派.八叶堂.付费同门切磋;
    if let Some(count) = calc_sparring_count(&data.member_challenged_times, is_paid) {
        同门切磋(d, count).await;
    }
}

/// 根据同门切磋剩余次数和付费开关计算可切磋次数
/// "0" 且未付费 → 1 次，"0" 且付费 → 2 次，"1" 且付费 → 1 次，其余 → 不打
fn calc_sparring_count(times: &str, is_paid: bool) -> Option<u8> {
    match (times, is_paid) {
        ("0", false) => Some(1),
        ("0", true) => Some(2),
        ("1", true) => Some(1),
        _ => None,
    }
}

async fn 木桩训练(d: &DaLeDou) {
    // 进入木桩训练
    let data: Response = match d.get("cmd=sect&op=trainwithnpc").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 同门切磋(d: &DaLeDou, mut count: u8) {
    while count > 0 {
        // 进入同门切磋
        let data: Response = match d.get("cmd=sect&op=trainwithmember").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        if data.result == "0" {
            count -= 1;
            continue;
        }

        if !兑换(d, "1249").await {
            return;
        }
    }
}

async fn 兑换(d: &DaLeDou, id: &str) -> bool {
    let cmd = format!("cmd=exchange&subtype=2&type={id}&times=1");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    data.result == "0"
}

async fn 五花堂(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct SectTask {
        result: String,
        msg: String,
        sect_id: String, // 门派id
        task: Vec<Task>, // 任务列表
    }

    #[derive(Deserialize)]
    struct Task {
        id: String,
        state: String,
        desc: String,
    }

    // 五花堂
    let data: SectTask = match d.get("cmd=sect_task").await {
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

    for item in &data.task {
        // 待领取或者已领取
        if item.state != "0" {
            continue;
        }

        match item.desc.as_str() {
            "进入华藏寺看一看" => 看一看(d, "cmd=sect_art").await,
            "进入伏虎寺看一看" => 看一看(d, "cmd=sect_trump").await,
            "进入金顶看一看" => 看一看(d, "cmd=sect&op=showcouncil").await,
            "进入八叶堂看一看" => 看一看(d, "cmd=sect&op=showtraining").await,
            "进入万年寺看一看" => 看一看(d, "cmd=sect&op=showincense").await,
            "与掌门人进行一次武艺切磋" => 切磋(d, "1").await,
            "与首座进行一次武艺切磋" => 切磋(d, "2").await,
            "与堂主进行一次武艺切磋" => 切磋(d, "3").await,
            "查看一名同门成员的资料" => 查看同门(d, &data.sect_id).await,
            "查看一名其他门派成员的资料" => 查看异门(d, &data.sect_id).await,
            "进行一次心法修炼" => 修炼(d).await,
            _ => {}
        }
    }

    // 五花堂
    let data: SectTask = match d.get("cmd=sect_task").await {
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

    for item in &data.task {
        // 未完成或者已领取
        if item.state != "1" {
            continue;
        }

        完成(d, &item.id).await;
    }
}

async fn 看一看(d: &DaLeDou, cmd: &str) {
    let _data: Response = match d.get(cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };
}

async fn 切磋(d: &DaLeDou, rank: &str) {
    #[derive(Deserialize)]
    struct ShowCouncil {
        result: String,
        msg: String,
        in_challenge_time: String, // 挑战时间
    }

    // 金顶
    let data: ShowCouncil = match d.get("cmd=sect&op=showcouncil").await {
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

    // 当前处于挑战时间
    if data.in_challenge_time != "0" {
        return;
    }

    // 切磋
    let cmd = format!("cmd=sect&op=trainingwithcouncil&rank={rank}&pos=1");
    let resp: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &resp.msg);
}

async fn 查看同门(d: &DaLeDou, sect_id: &str) {
    let Some(data) = view(d).await else {
        return;
    };

    for item in &data.info {
        if item.sect != "0" && item.sect == sect_id {
            查看(d, &item.uin).await;
            return;
        }
    }
}

async fn 查看异门(d: &DaLeDou, sect_id: &str) {
    let Some(data) = view(d).await else {
        return;
    };

    for item in &data.info {
        if item.sect != "0" && item.sect != sect_id {
            查看(d, &item.uin).await;
            return;
        }
    }
}

async fn 查看(d: &DaLeDou, uin: &str) {
    // 好友资料
    let cmd = format!("cmd=visit&puin={uin}&kind=1");
    let _data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };
}

#[derive(Deserialize)]
struct View {
    result: String,
    msg: String,
    info: Vec<Info>,
}

#[derive(Deserialize)]
struct Info {
    uin: String,
    sect: String,
}

async fn view(d: &DaLeDou) -> Option<View> {
    let cmd = format!("cmd=view&kind=1&sub=1&selfuin={}", d.qq());
    let data: View = match d.get(&cmd).await {
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

async fn 修炼(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct SectArt {
        result: String,
        msg: String,
        level: String,
        max_level: String,
    }

    for id in 101..=108 {
        // 心法
        let cmd = format!("cmd=sect_art&subtype=1&art_id={id}");
        let data: SectArt = match d.get(&cmd).await {
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

        // 已满级
        if data.level == data.max_level {
            continue;
        }

        // 修炼
        let ard_cmd = format!("cmd=sect_art&subtype=2&art_id={id}&times=1");
        let res: Response = match d.get(&ard_cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &res.msg);
        if res.result == "0" {
            return;
        }
    }
}

async fn 完成(d: &DaLeDou, id: &str) {
    // 完成
    let cmd = format!("cmd=sect_task&subtype=2&task_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_once() {
        assert_eq!(calc_sparring_count("0", false), Some(1));
    }

    #[test]
    fn test_paid_twice() {
        assert_eq!(calc_sparring_count("0", true), Some(2));
    }

    #[test]
    fn test_paid_second_once() {
        assert_eq!(calc_sparring_count("1", true), Some(1));
    }

    #[test]
    fn test_no_sparring() {
        assert_eq!(calc_sparring_count("1", false), None);
        assert_eq!(calc_sparring_count("2", true), None);
        assert_eq!(calc_sparring_count("2", false), None);
    }
}
