//! 帮派黄金联赛
//!
//! 参与防守、参战、领取轮次和排名奖励
//!
//! 按战力从小到大攻击

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "帮派黄金联赛";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        status: String,
        #[serde(default)]
        sign_up: String, // 是否已参与防守
        #[serde(default)]
        today_game: Vec<TodayGame>, // 对阵信息，当前仅用于判断是否结算日
    }

    #[derive(Deserialize)]
    struct TodayGame {}

    for _ in 0..2 {
        let data: Query = match d.get("cmd=factionleague&op=0").await {
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

        match data.status.as_str() {
            "1" => 领取轮次奖励(d).await,
            "2" => 领取排名奖励(d).await,
            "3" if data.sign_up == "0" => {
                参与防守(d).await;
                return;
            }
            "4" if !data.today_game.is_empty() && data.sign_up == "1" && is_win(d).await => {
                攻击(d).await;
                return;
            }
            _ => {}
        }
    }
}

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

async fn 领取轮次奖励(d: &DaLeDou) {
    // 领取奖励
    let data: Response = match d.get("cmd=factionleague&op=5").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取排名奖励(d: &DaLeDou) {
    // 领取奖励
    let data: Response = match d.get("cmd=factionleague&op=7").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 参与防守(d: &DaLeDou) {
    // 参与防守
    let data: Response = match d.get("cmd=factionleague&op=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn is_win(d: &DaLeDou) -> bool {
    #[derive(Deserialize)]
    struct FightRecord {
        result: String,
        msg: String,
        fight_list: Vec<FightList>,
    }

    #[derive(Deserialize)]
    struct FightList {
        uin: String,
        is_win: String, // 胜负
    }

    // 战斗记录
    let data: FightRecord = match d.get("cmd=factionleague&op=10").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return false;
    }

    for item in &data.fight_list {
        if item.uin != d.qq() {
            continue;
        }

        // 自己已阵亡
        if item.is_win == "0" {
            return false;
        }
    }

    true
}

async fn 攻击(d: &DaLeDou) {
    let Some(data) = 参战(d).await else {
        return;
    };

    let targets = sort_defense_list(data.defense_list);
    if targets.is_empty() {
        return;
    }

    for item in targets {
        // 攻击
        let cmd = format!("cmd=factionleague&op=4&opp_uin={}", item.uin);
        let resp: Response = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &resp.msg);
        if resp.msg.starts_with("更换一名") {
            continue;
        }
        if resp.result != "0" {
            return;
        }

        if !resp.msg.starts_with("勇士，恭喜您战胜") {
            return;
        }
    }
}

/// 按战力升序排列存活目标（cur_hp != "0"），解析失败的目标丢弃
fn sort_defense_list(defenses: Vec<DefenseList>) -> Vec<DefenseList> {
    let mut targets: Vec<(f64, DefenseList)> = defenses
        .into_iter()
        .filter(|d| d.cur_hp != "0")
        .filter_map(|d| {
            let power: f64 = d.atk_power.parse().ok()?;
            Some((power, d))
        })
        .collect();
    targets.sort_by(|a, b| a.0.total_cmp(&b.0));
    targets.into_iter().map(|(_, d)| d).collect()
}

#[derive(Deserialize)]
struct CanZhan {
    result: String,
    msg: String,
    defense_list: Vec<DefenseList>,
}

#[derive(Deserialize)]
struct DefenseList {
    uin: String,
    cur_hp: String,
    atk_power: String,
}

async fn 参战(d: &DaLeDou) -> Option<CanZhan> {
    // 参战
    let data: CanZhan = match d.get("cmd=factionleague&op=2").await {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn def(uin: &str, cur_hp: &str, atk_power: &str) -> DefenseList {
        DefenseList {
            uin: uin.to_string(),
            cur_hp: cur_hp.to_string(),
            atk_power: atk_power.to_string(),
        }
    }

    #[test]
    fn test_sort_by_power_ascending() {
        let list = vec![
            def("3", "100", "300.0"),
            def("1", "100", "100.0"),
            def("2", "100", "200.0"),
        ];
        let result = sort_defense_list(list);
        let uins: Vec<&str> = result.iter().map(|d| d.uin.as_str()).collect();
        assert_eq!(uins, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_filter_dead() {
        let list = vec![
            def("1", "0", "100.0"),
            def("2", "100", "200.0"),
            def("3", "0", "300.0"),
        ];
        let result = sort_defense_list(list);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].uin, "2");
    }

    #[test]
    fn test_filter_invalid_power() {
        let list = vec![def("1", "100", "abc"), def("2", "100", "200.0")];
        let result = sort_defense_list(list);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].uin, "2");
    }

    #[test]
    fn test_all_dead_or_invalid() {
        let list = vec![def("1", "0", "100.0"), def("2", "100", "abc")];
        assert!(sort_defense_list(list).is_empty());
    }
}
