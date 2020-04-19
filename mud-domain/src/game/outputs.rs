use crate::game::domain::Dir;
use crate::game::item::ItemId;
use crate::game::location::LocationId;
use crate::game::mob::{Damage, MobId};
use crate::game::room::RoomId;
use commons::ObjId;

#[derive(Debug, Clone)]
pub enum Output {
    LookedAt {
        target: ObjId,
    },
    Examined {
        target: ObjId,
    },
    MovedDir {
        from: LocationId,
        to: LocationId,
        from_dir: Dir,
    },
    Enter {
        at: ObjId,
        to: RoomId,
    },
    Rested,
    Stand,
    StatsReport,
    Pick {
        item: ItemId,
    },
    Equip {
        item: ItemId,
    },
    Remove {
        item: ItemId,
    },
    Kill {
        mob_id: MobId,
    },
    Hit {
        mob_id: MobId,
        amount: Damage,
    },
    ReceiveDamage {
        amount: Damage,
    },
    Said {
        msg: String,
    },
    Landed,
    Launch,
    Bough {
        target: ObjId,
    },
    Sell {
        target: ObjId,
    },
}
