use crate::game::domain::Dir;
use crate::game::item::ItemId;
use crate::game::location::LocationId;
use crate::game::mob::{Damage, MobId};
use crate::game::room::RoomId;
use commons::ObjId;
use serde::export::Formatter;

pub const O_PLAIN: &str = "^pl";
pub const O_RESET: &str = "^rs";
pub const O_LITERAL: &str = "^li";

pub enum OMarker {
    Plain,
    Reset,
    Literal,
}

impl OMarker {
    pub fn list() -> Vec<OMarker> {
        vec![OMarker::Plain, OMarker::Literal, OMarker::Reset]
    }

    pub fn id(&self) -> &'static str {
        match self {
            OMarker::Plain => "\\p",
            OMarker::Reset => "\\r",
            OMarker::Literal => "\\l",
        }
    }
}

impl std::fmt::Display for OMarker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

#[derive(Debug, Clone)]
pub enum Output {
    Private {
        mob_id: MobId,
        msg: String,
    },

    Broadcast {
        /// usually the mob that originate the message
        exclude: Option<MobId>,
        /// RoomId or ZoneId, all children mobs will receive the message
        location_id: LocationId,
        /// recursive search for mobs to send message
        recursive: bool,
        msg: String,
    },
}

#[derive(Debug, Clone)]
pub struct Outputs {
    list: Vec<Output>,
}

impl Outputs {
    pub fn new() -> Self {
        Outputs { list: vec![] }
    }

    pub fn take(&mut self) -> Vec<Output> {
        std::mem::replace(&mut self.list, vec![])
    }

    /// For all mobs recursive inside the location
    pub fn broadcast_all(&mut self, exclude: Option<ObjId>, location_id: ObjId, msg: String) {
        self.list.push(Output::Broadcast {
            exclude,
            location_id,
            msg,
            recursive: true,
        })
    }

    /// For all mobs in current location
    pub fn broadcast(&mut self, exclude: Option<ObjId>, location_id: ObjId, msg: String) {
        self.list.push(Output::Broadcast {
            exclude,
            location_id,
            msg,
            recursive: false,
        })
    }

    /// Just to a specific mob
    pub fn private(&mut self, mob_id: MobId, msg: String) {
        self.list.push(Output::Private { mob_id, msg })
    }
}
