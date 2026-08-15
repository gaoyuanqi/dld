//! 乐斗好友NPC、帮友NPC、侠侣NPC、结拜NPC、师徒妻拜
//!
//! 跳过已乐斗

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "乐斗";

pub async fn run(d: &DaLeDou) {
    let Some(view_data) = view(d).await else {
        return;
    };

    let Some(visit_data) = visit(d).await else {
        return;
    };

    let lilian: u32 = visit_data.baseinfo.lilian.parse().unwrap_or(0);
    好友_npc(d, &view_data, lilian).await;

    if visit_data.baseinfo.factionid != "0" {
        帮友_npc(d).await;
    }

    if visit_data.marry.parternuin != "0" {
        // 大色魔、四姑娘、曾小三
        侠侣_npc(d, &[13, 15, 152]).await;
    }

    if !visit_data.brother.brother_list.is_empty() {
        let b: u32 = visit_data.brother.brotherhood.parse().unwrap_or(0);
        let mut npcs = vec![17u8];
        // 强盗
        if b >= 800 {
            npcs.push(18);
        }
        // 盗圣
        if b >= 2500 {
            npcs.push(153);
        }
        侠侣_npc(d, &npcs).await;
    }

    师傅(d, &visit_data).await;
    徒弟(d, &visit_data).await;
    夫妻(d, &visit_data).await;
    结拜(d, &visit_data).await;
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
    enable: String,
    faceid: Option<String>, // 玩家才有该字段
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

#[derive(Deserialize)]
struct Visit {
    result: String,
    msg: String,
    baseinfo: BaseInfo,
    shifu: ShiFu,
    tudi: Vec<TuDi>,
    marry: Marry,
    brother: Brother,
}

#[derive(Deserialize)]
struct BaseInfo {
    lilian: String,    // 玩家等级
    factionid: String, // 帮派id
}

#[derive(Deserialize)]
struct ShiFu {
    #[serde(default)]
    uin: String,
}

#[derive(Deserialize)]
struct TuDi {
    uin: String,
}

#[derive(Deserialize)]
struct Marry {
    #[serde(default)]
    parternuin: String,
}

#[derive(Deserialize)]
struct Brother {
    #[serde(default, rename = "brotherList")]
    brother_list: Vec<BrotherList>,
    #[serde(default)]
    brotherhood: String, // 义气度
}

#[derive(Deserialize)]
struct BrotherList {
    brotheruin: String,
}

async fn visit(d: &DaLeDou) -> Option<Visit> {
    let cmd = format!("cmd=visit&puin={}&kind=1", d.qq());
    let data: Visit = match d.get(&cmd).await {
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

#[derive(Deserialize)]
struct UinVisit {
    result: String,
    msg: String,
    #[serde(default)]
    uin: String,
    baseinfo: UinBaseInfo,
}

#[derive(Deserialize)]
struct UinBaseInfo {
    #[serde(default)]
    fightflag: String, // 是否乐斗该玩家
}

async fn uin_visit(d: &DaLeDou, uin: &str) -> Option<UinVisit> {
    let cmd = format!("cmd=visit&puin={uin}&kind=1");
    let data: UinVisit = match d.get(&cmd).await {
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

async fn 好友_npc(d: &DaLeDou, data: &View, lilian: u32) {
    for item in &data.info {
        if item.faceid.is_some() {
            break;
        }
        if item.enable == "1" && npc_lv_ok(&item.uin, lilian) {
            乐斗(d, &format!("cmd=fight&puin={}", item.uin)).await;
        }
    }
}

// NPC 等级门槛
fn npc_lv_ok(uin: &str, lilian: u32) -> bool {
    match uin {
        "9" => lilian >= 20,    // 乐斗剑君
        "11" => lilian >= 30,   // 月敏妹妹
        "12" => lilian >= 40,   // 俊猴王
        "16" => lilian >= 50,   // 乐斗程管
        "33" => lilian >= 60,   // 金毛鹅王
        "155" => lilian >= 90,  // 一灯大师
        "156" => lilian >= 100, // 黄药师
        _ => true,              // 其他 NPC 无门槛
    }
}

async fn 帮友_npc(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct ViewMember {
        result: String,
        msg: String,
        #[serde(default)]
        list: Vec<List>,
    }

    #[derive(Deserialize)]
    struct List {
        uin: String,
        fight: String,
        faceid: Option<String>, // 玩家才有该字段
    }

    let data: ViewMember = match d.get("cmd=viewmember").await {
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

    // 跳过列表第一个（守护神）
    for item in data.list.iter().skip(1) {
        if item.faceid.is_some() {
            break;
        }
        if item.fight == "1" {
            乐斗(d, &format!("cmd=fight&uin={}", item.uin)).await;
        }
    }
}

async fn 侠侣_npc(d: &DaLeDou, uin: &[u8]) {
    #[derive(Deserialize)]
    struct ViewNpc {
        result: String,
        msg: String,
        baseinfo: BaseInfo,
    }

    #[derive(Deserialize)]
    struct BaseInfo {
        #[serde(default)]
        fightflag: String, // 乐斗状态
    }

    for u in uin {
        let cmd = format!("cmd=viewnpc&id={u}");
        let data: ViewNpc = match d.get(&cmd).await {
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

        // 未乐斗
        if data.baseinfo.fightflag == "1" {
            乐斗(d, &format!("cmd=fight&puin={u}")).await;
        }
    }
}

async fn 师傅(d: &DaLeDou, visit_data: &Visit) {
    // 没有师傅
    if visit_data.shifu.uin.is_empty() {
        return;
    }

    let Some(data) = uin_visit(d, &visit_data.shifu.uin).await else {
        return;
    };

    // 未乐斗
    if data.baseinfo.fightflag == "0" {
        乐斗(d, &format!("cmd=fight&puin={}", data.uin)).await;
    }
}

async fn 徒弟(d: &DaLeDou, visit_data: &Visit) {
    for item in &visit_data.tudi {
        let Some(data) = uin_visit(d, &item.uin).await else {
            return;
        };

        // 未乐斗
        if data.baseinfo.fightflag == "0" {
            乐斗(d, &format!("cmd=fight&puin={}", data.uin)).await;
        }
    }
}

async fn 夫妻(d: &DaLeDou, visit_data: &Visit) {
    // 没有结婚
    if visit_data.marry.parternuin == "0" {
        return;
    }

    let Some(data) = uin_visit(d, &visit_data.marry.parternuin).await else {
        return;
    };

    // 未乐斗
    if data.baseinfo.fightflag == "0" {
        乐斗(d, &format!("cmd=fight&puin={}", data.uin)).await;
    }
}

async fn 结拜(d: &DaLeDou, visit_data: &Visit) {
    for item in &visit_data.brother.brother_list {
        let Some(data) = uin_visit(d, &item.brotheruin).await else {
            return;
        };

        // 未乐斗
        if data.baseinfo.fightflag == "0" {
            乐斗(d, &format!("cmd=fight&puin={}", data.uin)).await;
        }
    }
}

async fn 乐斗(d: &DaLeDou, cmd: &str) {
    #[derive(Deserialize)]
    struct Fight {
        result: String,
        msg: String,
        #[serde(default)]
        repid: String,
    }

    // 乐斗
    let data: Fight = match d.get(cmd).await {
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

    if !data.repid.is_empty() {
        乐斗记录(d, &data.repid).await;
    }
}

async fn 乐斗记录(d: &DaLeDou, repid: &str) {
    #[derive(Deserialize)]
    struct View {
        result: String,
        msg: String,
        info: Vec<ViewInfo>,
    }

    #[derive(Deserialize)]
    struct ViewInfo {
        url: String,
        desc: String,
    }

    let cmd = format!("cmd=view&kind=2&sub=1&selfuin={}", d.qq());
    let data: View = match d.get(&cmd).await {
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

    for item in &data.info {
        if item.url == repid {
            d.log(TASK, &item.desc);
            return;
        }
    }
}
