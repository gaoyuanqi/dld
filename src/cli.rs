//! 命令行解析：每个子命令对应一个任务，支持 `-q` 指定单个 QQ
//!
//! # 示例
//!
//! ```text
//! dld 代玩              # 执行所有任务
//! dld 代玩 -q 123456    # 指定 QQ 执行
//! dld 帮派商会           # 单独执行帮派商会
//! ```

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

use crate::app::App;
use crate::dw::tasks::Task;

#[derive(Parser)]
#[command(
    version,
    about,
    after_help = r#"
所有任务命令（代玩、乐斗、武林等）都支持 -q 选项，用于指定单个 QQ 号。

示例：
  dld 标准目录
  dld 同步配置
  dld 登记 "openId=xxx; accessToken=yyy; newuin=123456"
  dld 注销 123456
  dld 代玩
  dld 代玩 -q 123456
  dld 邪神秘宝
  dld 邪神秘宝 -q 123456
"#
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// 共享的 -q 参数，用于指定单个 QQ 号
#[derive(clap::Args)]
struct QqArg {
    /// 指定单个 QQ 号
    #[arg(short, long)]
    qq: Option<String>,
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
enum Command {
    /// 添加或更新Cookie、创建配置文件
    登记 {
        /// 格式："openId=...; accessToken=...; newuin=..."
        cookie: String,
    },
    /// 从标准目录移除Cookie、配置、日志
    注销 { qq: String },
    /// 打印程序数据存储目录
    标准目录,
    /// 同步配置文件到最新格式，适配新版本
    同步配置,
    /// 执行下面列出的所有任务
    代玩 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 分享、斗神塔挑战、领奖、重置
    分享 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 乐斗BOOS、师徒妻拜
    乐斗 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 报名
    武林 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 报名、助威、领斗币
    结拜 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 报名
    侠侣 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 报名
    群侠 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 挑战、领奖、开启副本
    矿洞 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 掠夺、 领奖、报名、领取胜负奖励
    掠夺 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 试炼、挑战、报名、领奖、领取排行奖励
    踢馆 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取、领取许愿奖励、许愿
    许愿 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 乐斗掉落佣兵碎片BOSS
    历练 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 乐斗、领取奖励
    幻境 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 万年寺、八叶堂、五花堂
    门派 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 试炼、助威、兑换、领奖
    会武 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 使用锦囊类、以宝箱、食盒结尾的物品
    背包 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 免费挑战、领取奖励、兑换
    竞技场 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 请猴王扫荡最高场景
    十二宫 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 免费攻占、每日奖励
    抢地盘 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 优先太玄经、玄铁令
    侠客岛 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取、免费温养
    世界树 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    每日奖励 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 打开
    每日宝箱 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 高级和极品免费抽奖
    邪神秘宝 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 免费挑战、领取段位奖励
    华山论剑 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 报名北派、领奖、征战
    巅峰之战 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取奖励、护送、拦截
    镖行天下 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 报名、领奖
    群雄逐鹿 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 准备完成进入战斗
    画卷迷踪 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    任务 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 转动转盘
    帮派祭坛 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 普通旅行、梦幻旅行、领取
    梦想之旅 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领奖、助威、攻占
    问鼎天下 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取、交易、兑换
    帮派商会 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领奖、报名、竞猜
    武林盟主 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    全民乱斗 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取奖励、奇遇、领取食盒
    侠士客栈 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 副本、兑换
    江湖长梦 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取巡礼、秘境
    深渊之潮 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 八卦、挑战、领奖、兑换
    时空遗迹 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 报名、挑战、领奖、兑换
    龙凰之境 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 任务、供奉5次、帮战
    我的帮派 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 报名、领取段位奖励、免费挑战、兑换
    门派邀请赛 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 攻击、领取岛屿和节点奖励
    帮派远征军 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 优先报名单排、领奖
    飞升大作战 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取礼包
    今日活跃度 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 参与防守、参战、领奖
    帮派黄金联赛 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取奖励、接受任务
    任务派遣中心 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    领取徒弟经验 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取、寻访
    仙武修真 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取、占卜
    乐斗黄历 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    器魂附魔 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 周四微信兑换
    兑换码 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取、翻牌
    激运牌 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 单双单双单
    猜单双 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 免费抓取一次
    娃娃机 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取淬火结晶*1
    乐斗驿站 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 免费幸运抽奖
    神魔转盘 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取登录和充值礼包
    登录有礼 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取每日礼包
    徽章战令 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 免费随机、挑战
    职业挑战 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取当天和累计奖励
    斗境探秘 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 三魂和七魄免费抽奖
    深渊秘宝 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取50、80活跃礼包
    活跃礼包 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取积分、一键领取、兑换
    乐斗游记 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    浩劫宝箱 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    周周礼包 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    好礼提升 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 砸金蛋
    幸运金蛋 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    元武登高 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 点单
    乐斗菜单 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 转动转盘
    幸运转盘 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    大侠回归 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 兑换
    登录商店 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 领取
    豪侠出世 {
        #[command(flatten)]
        qq: QqArg,
    },
    /// 打印帮助信息
    #[command(name = "help")]
    帮助,
}

pub async fn parse_args() -> Result<()> {
    let app = App::init()?;
    let cli = Cli::parse();

    match cli.command {
        Command::登记 { cookie } => app.register(&cookie).await?,
        Command::注销 { qq } => app.unregister(&qq)?,
        Command::标准目录 => app.print_std_dirs(),
        Command::同步配置 => app.update_config()?,
        Command::代玩 { qq } => app.run_all_task(qq.qq).await?,
        Command::分享 { qq } => app.run_task(Task::分享, qq.qq).await?,
        Command::乐斗 { qq } => app.run_task(Task::乐斗, qq.qq).await?,
        Command::武林 { qq } => app.run_task(Task::武林, qq.qq).await?,
        Command::结拜 { qq } => app.run_task(Task::结拜, qq.qq).await?,
        Command::侠侣 { qq } => app.run_task(Task::侠侣, qq.qq).await?,
        Command::群侠 { qq } => app.run_task(Task::群侠, qq.qq).await?,
        Command::矿洞 { qq } => app.run_task(Task::矿洞, qq.qq).await?,
        Command::掠夺 { qq } => app.run_task(Task::掠夺, qq.qq).await?,
        Command::踢馆 { qq } => app.run_task(Task::踢馆, qq.qq).await?,
        Command::许愿 { qq } => app.run_task(Task::许愿, qq.qq).await?,
        Command::历练 { qq } => app.run_task(Task::历练, qq.qq).await?,
        Command::幻境 { qq } => app.run_task(Task::幻境, qq.qq).await?,
        Command::门派 { qq } => app.run_task(Task::门派, qq.qq).await?,
        Command::会武 { qq } => app.run_task(Task::会武, qq.qq).await?,
        Command::背包 { qq } => app.run_task(Task::背包, qq.qq).await?,
        Command::竞技场 { qq } => app.run_task(Task::竞技场, qq.qq).await?,
        Command::十二宫 { qq } => app.run_task(Task::十二宫, qq.qq).await?,
        Command::抢地盘 { qq } => app.run_task(Task::抢地盘, qq.qq).await?,
        Command::侠客岛 { qq } => app.run_task(Task::侠客岛, qq.qq).await?,
        Command::世界树 { qq } => app.run_task(Task::世界树, qq.qq).await?,
        Command::每日奖励 { qq } => app.run_task(Task::每日奖励, qq.qq).await?,
        Command::每日宝箱 { qq } => app.run_task(Task::每日宝箱, qq.qq).await?,
        Command::邪神秘宝 { qq } => app.run_task(Task::邪神秘宝, qq.qq).await?,
        Command::华山论剑 { qq } => app.run_task(Task::华山论剑, qq.qq).await?,
        Command::巅峰之战 { qq } => app.run_task(Task::巅峰之战, qq.qq).await?,
        Command::镖行天下 { qq } => app.run_task(Task::镖行天下, qq.qq).await?,
        Command::群雄逐鹿 { qq } => app.run_task(Task::群雄逐鹿, qq.qq).await?,
        Command::画卷迷踪 { qq } => app.run_task(Task::画卷迷踪, qq.qq).await?,
        Command::任务 { qq } => app.run_task(Task::任务, qq.qq).await?,
        Command::帮派祭坛 { qq } => app.run_task(Task::帮派祭坛, qq.qq).await?,
        Command::梦想之旅 { qq } => app.run_task(Task::梦想之旅, qq.qq).await?,
        Command::问鼎天下 { qq } => app.run_task(Task::问鼎天下, qq.qq).await?,
        Command::帮派商会 { qq } => app.run_task(Task::帮派商会, qq.qq).await?,
        Command::武林盟主 { qq } => app.run_task(Task::武林盟主, qq.qq).await?,
        Command::全民乱斗 { qq } => app.run_task(Task::全民乱斗, qq.qq).await?,
        Command::侠士客栈 { qq } => app.run_task(Task::侠士客栈, qq.qq).await?,
        Command::江湖长梦 { qq } => app.run_task(Task::江湖长梦, qq.qq).await?,
        Command::深渊之潮 { qq } => app.run_task(Task::深渊之潮, qq.qq).await?,
        Command::时空遗迹 { qq } => app.run_task(Task::时空遗迹, qq.qq).await?,
        Command::龙凰之境 { qq } => app.run_task(Task::龙凰之境, qq.qq).await?,
        Command::我的帮派 { qq } => app.run_task(Task::我的帮派, qq.qq).await?,
        Command::门派邀请赛 { qq } => app.run_task(Task::门派邀请赛, qq.qq).await?,
        Command::帮派远征军 { qq } => app.run_task(Task::帮派远征军, qq.qq).await?,
        Command::飞升大作战 { qq } => app.run_task(Task::飞升大作战, qq.qq).await?,
        Command::今日活跃度 { qq } => app.run_task(Task::今日活跃度, qq.qq).await?,
        Command::帮派黄金联赛 { qq } => app.run_task(Task::帮派黄金联赛, qq.qq).await?,
        Command::任务派遣中心 { qq } => app.run_task(Task::任务派遣中心, qq.qq).await?,
        Command::领取徒弟经验 { qq } => app.run_task(Task::领取徒弟经验, qq.qq).await?,
        Command::仙武修真 { qq } => app.run_task(Task::仙武修真, qq.qq).await?,
        Command::乐斗黄历 { qq } => app.run_task(Task::乐斗黄历, qq.qq).await?,
        Command::器魂附魔 { qq } => app.run_task(Task::器魂附魔, qq.qq).await?,
        Command::兑换码 { qq } => app.run_task(Task::兑换码, qq.qq).await?,
        Command::激运牌 { qq } => app.run_task(Task::激运牌, qq.qq).await?,
        Command::猜单双 { qq } => app.run_task(Task::猜单双, qq.qq).await?,
        Command::娃娃机 { qq } => app.run_task(Task::娃娃机, qq.qq).await?,
        Command::乐斗驿站 { qq } => app.run_task(Task::乐斗驿站, qq.qq).await?,
        Command::神魔转盘 { qq } => app.run_task(Task::神魔转盘, qq.qq).await?,
        Command::登录有礼 { qq } => app.run_task(Task::登录有礼, qq.qq).await?,
        Command::徽章战令 { qq } => app.run_task(Task::徽章战令, qq.qq).await?,
        Command::职业挑战 { qq } => app.run_task(Task::职业挑战, qq.qq).await?,
        Command::斗境探秘 { qq } => app.run_task(Task::斗境探秘, qq.qq).await?,
        Command::深渊秘宝 { qq } => app.run_task(Task::深渊秘宝, qq.qq).await?,
        Command::活跃礼包 { qq } => app.run_task(Task::活跃礼包, qq.qq).await?,
        Command::乐斗游记 { qq } => app.run_task(Task::乐斗游记, qq.qq).await?,
        Command::浩劫宝箱 { qq } => app.run_task(Task::浩劫宝箱, qq.qq).await?,
        Command::周周礼包 { qq } => app.run_task(Task::周周礼包, qq.qq).await?,
        Command::好礼提升 { qq } => app.run_task(Task::好礼提升, qq.qq).await?,
        Command::幸运金蛋 { qq } => app.run_task(Task::幸运金蛋, qq.qq).await?,
        Command::元武登高 { qq } => app.run_task(Task::元武登高, qq.qq).await?,
        Command::乐斗菜单 { qq } => app.run_task(Task::乐斗菜单, qq.qq).await?,
        Command::幸运转盘 { qq } => app.run_task(Task::幸运转盘, qq.qq).await?,
        Command::大侠回归 { qq } => app.run_task(Task::大侠回归, qq.qq).await?,
        Command::登录商店 { qq } => app.run_task(Task::登录商店, qq.qq).await?,
        Command::豪侠出世 { qq } => app.run_task(Task::豪侠出世, qq.qq).await?,
        Command::帮助 => {
            Cli::command().print_help()?;
        }
    }

    Ok(())
}
