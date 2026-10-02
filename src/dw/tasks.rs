mod bai_pai_ji_tan;
mod bang_pai_shang_hui;
mod bei_bao;
mod biao_xing_tian_xia;
mod cai_dan_shuang;
mod da_xia_hui_gui;
mod deng_lu_shang_dian;
mod deng_lu_you_li;
mod dian_feng_zhi_zhan;
mod dou_jing_tan_mi;
mod dui_huan_ma;
mod fei_sheng;
mod fen_xiang;
mod gua_gua_ka;
mod hao_jie_bao_xiang;
mod hao_li_ti_sheng;
mod hao_xia_chu_shi;
mod hua_juan_mi_zong;
mod hua_shan_lun_jian;
mod huan_jing;
mod hui_wu;
mod hui_zhang_zhan_ling;
mod huo_yue_li_bao;
mod ji_li_dui;
mod ji_yun_pai;
mod jiang_hu_chang_meng;
mod jie_bai;
mod jin_qiu_hui_kui;
mod jin_ri_huo_yue_du;
mod jing_ji_chang;
mod kai_xue_ji;
mod kuang_dong;
mod le_dou;
mod le_dou_cai_dan;
mod le_dou_huang_li;
mod le_dou_yi_zhan;
mod le_dou_you_ji;
mod li_lian;
mod lian_sai;
mod ling_qu_tu_di;
mod long_huang_zhi_jing;
mod lve_duo;
mod mei_ri_bao_xiang;
mod mei_ri_jiang_li;
mod men_pai;
mod men_pai_yao_qing_sai;
mod meng_xiang_zhi_lv;
mod mi_ji_feng_yin;
mod qi_hun_fu_mo;
mod qiang_di_pan;
mod quan_min_luan_dou;
mod qun_xia;
mod qun_xiong_zhu_lu;
mod ren_wu;
mod ren_wu_pai_qian;
mod shen_mo_zhuan_pan;
mod shen_yuan_mi_bao;
mod shen_yuan_zhi_chao;
mod shi_er_gong;
mod shi_jie_shu;
mod shi_kong_yi_ji;
mod ti_guan;
mod wa_wa_ji;
mod wen_ding_tian_xia;
mod wo_de_bang_pai;
mod wu_lin;
mod wu_lin_meng_zhu;
mod xia_ke_dao;
mod xia_lv;
mod xia_shi_ke_zhan;
mod xian_wu_xiu_zhen;
mod xie_shen_mi_bao;
mod xing_yun_jin_dan;
mod xing_yun_zhuan_pan;
mod xu_yuan;
mod yuan_wu_deng_gao;
mod yuan_zheng;
mod zhi_ye_tiao_zhan;
mod zhong_qiu_li_he;
mod zhou_zhou_li_bao;

use crate::dw::daledou::DaLeDou;

/// 从「任务名 => 任务模块」列表生成任务枚举、全部任务列表和分发函数
///
/// 新增任务只需在 `tasks!` 调用中加一行，三处登记（枚举变体、
/// `all()` 列表、`run_task` 分发）自动同步，避免漏登记
macro_rules! tasks {
    ($($任务:ident => $模块:ident),* $(,)?) => {
        #[derive(Clone, Debug)]
        pub enum Task {
            $($任务),*
        }

        impl Task {
            /// 返回全部任务列表（内部使用）
            pub fn all() -> &'static [Task] {
                &[$(Task::$任务),*]
            }
        }

        /// 运行单个任务
        pub async fn run_task(d: &DaLeDou, name: &Task) {
            match name {
                $(Task::$任务 => $模块::run(d).await),*
            }
        }
    };
}

tasks! {
    分享 => fen_xiang,
    乐斗 => le_dou,
    武林 => wu_lin,
    结拜 => jie_bai,
    侠侣 => xia_lv,
    群侠 => qun_xia,
    矿洞 => kuang_dong,
    掠夺 => lve_duo,
    踢馆 => ti_guan,
    许愿 => xu_yuan,
    历练 => li_lian,
    幻境 => huan_jing,
    门派 => men_pai,
    会武 => hui_wu,
    背包 => bei_bao,
    竞技场 => jing_ji_chang,
    十二宫 => shi_er_gong,
    抢地盘 => qiang_di_pan,
    侠客岛 => xia_ke_dao,
    世界树 => shi_jie_shu,
    每日奖励 => mei_ri_jiang_li,
    每日宝箱 => mei_ri_bao_xiang,
    邪神秘宝 => xie_shen_mi_bao,
    华山论剑 => hua_shan_lun_jian,
    巅峰之战 => dian_feng_zhi_zhan,
    镖行天下 => biao_xing_tian_xia,
    群雄逐鹿 => qun_xiong_zhu_lu,
    画卷迷踪 => hua_juan_mi_zong,
    任务 => ren_wu,
    帮派祭坛 => bai_pai_ji_tan,
    梦想之旅 => meng_xiang_zhi_lv,
    问鼎天下 => wen_ding_tian_xia,
    帮派商会 => bang_pai_shang_hui,
    武林盟主 => wu_lin_meng_zhu,
    全民乱斗 => quan_min_luan_dou,
    侠士客栈 => xia_shi_ke_zhan,
    江湖长梦 => jiang_hu_chang_meng,
    深渊之潮 => shen_yuan_zhi_chao,
    时空遗迹 => shi_kong_yi_ji,
    龙凰之境 => long_huang_zhi_jing,
    我的帮派 => wo_de_bang_pai,
    门派邀请赛 => men_pai_yao_qing_sai,
    帮派远征军 => yuan_zheng,
    飞升大作战 => fei_sheng,
    今日活跃度 => jin_ri_huo_yue_du,
    帮派黄金联赛 => lian_sai,
    任务派遣中心 => ren_wu_pai_qian,
    领取徒弟经验 => ling_qu_tu_di,
    仙武修真 => xian_wu_xiu_zhen,
    乐斗黄历 => le_dou_huang_li,
    器魂附魔 => qi_hun_fu_mo,
    兑换码 => dui_huan_ma,
    激运牌 => ji_yun_pai,
    猜单双 => cai_dan_shuang,
    娃娃机 => wa_wa_ji,
    开学季 => kai_xue_ji,
    刮刮卡 => gua_gua_ka,
    吉利兑 => ji_li_dui,
    乐斗驿站 => le_dou_yi_zhan,
    神魔转盘 => shen_mo_zhuan_pan,
    登录有礼 => deng_lu_you_li,
    徽章战令 => hui_zhang_zhan_ling,
    职业挑战 => zhi_ye_tiao_zhan,
    斗境探秘 => dou_jing_tan_mi,
    深渊秘宝 => shen_yuan_mi_bao,
    活跃礼包 => huo_yue_li_bao,
    乐斗游记 => le_dou_you_ji,
    浩劫宝箱 => hao_jie_bao_xiang,
    周周礼包 => zhou_zhou_li_bao,
    好礼提升 => hao_li_ti_sheng,
    幸运金蛋 => xing_yun_jin_dan,
    元武登高 => yuan_wu_deng_gao,
    乐斗菜单 => le_dou_cai_dan,
    幸运转盘 => xing_yun_zhuan_pan,
    金秋回馈 => jin_qiu_hui_kui,
    大侠回归 => da_xia_hui_gui,
    登录商店 => deng_lu_shang_dian,
    豪侠出世 => hao_xia_chu_shi,
    秘籍封印 => mi_ji_feng_yin,
    中秋礼盒 => zhong_qiu_li_he,
}

#[cfg(test)]
mod tests {
    use super::*;

    // 任务列表数量：锁定登记完整性，防止重构时漏任务
    #[test]
    fn test_task_all_count() {
        assert_eq!(Task::all().len(), 80);
    }

    // 任务列表不允许重复
    #[test]
    fn test_task_all_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for task in Task::all() {
            assert!(seen.insert(format!("{task:?}")), "任务重复登记：{task:?}");
        }
    }
}
