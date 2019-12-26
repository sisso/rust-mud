use crate::errors::Result;
use commons::{DeltaTime, ObjId, PlayerId};
use logs::*;
use crate::game::loader::Loader;
use crate::game::{Outputs, spawn, mob, item, crafts_system};
use crate::game::config::Config;
use crate::game::domain::{GameTime, PlayerCtx, MobCtx};
use crate::game::obj::Objects;
use crate::game::player::PlayerRepository;
use crate::game::mob::{MobRepository, MobId};
use crate::game::item::ItemRepository;
use crate::game::room::RoomRepository;
use crate::game::spawn::Spawns;
use crate::game::location::Locations;
use crate::game::equip::Equips;
use crate::game::tags::Tags;
use crate::game::labels::Labels;
use crate::game::crafts::Ships;
use crate::game::surfaces::Surfaces;
use crate::game::astro_bodies::AstroBodies;
use crate::game::pos::PosRepo;
use crate::game::surfaces_object::SurfaceObjects;
use crate::game::vendors::Vendors;
use crate::game::prices::Prices;

/// Until standardize it or remove is not defined, should be used only in System, not in
/// command handling
pub struct Ctx<'a> {
    pub container: &'a mut Container,
    pub outputs: &'a mut dyn Outputs,
}

pub struct Container {
    pub config: Config,
    pub time: GameTime,
    pub objects: Objects,
    pub players: PlayerRepository,
    pub mobs: MobRepository,
    pub items: ItemRepository,
    pub rooms: RoomRepository,
    pub spawns: Spawns,
    pub locations: Locations,
    pub equips: Equips,
    pub tags: Tags,
    pub labels: Labels,
    pub ship: Ships,
    pub sectors: Surfaces,
    pub astro_bodies: AstroBodies,
    pub pos: PosRepo,
    pub surface_objects: SurfaceObjects,
    pub loader: Loader,
    pub vendors: Vendors,
    pub prices: Prices,
}

impl Container {
    pub fn new() -> Self {
        Container {
            config: Config::new(),
            time: GameTime::new(),
            objects: Objects::new(),
            players: PlayerRepository::new(),
            mobs: MobRepository::new(),
            items: ItemRepository::new(),
            rooms: RoomRepository::new(),
            spawns: Spawns::new(),
            locations: Locations::new(),
            equips: Equips::new(),
            tags: Tags::new(),
            labels: Labels::new(),
            ship: Ships::new(),
            sectors: Surfaces::new(),
            astro_bodies: AstroBodies::new(),
            pos: PosRepo::new(),
            surface_objects: SurfaceObjects::new(),
            loader: Loader::new(),
            vendors: Vendors::new(),
            prices: Prices::new(),
        }
    }

    // TODO: use macro trait or some more generic way to remove all references
    pub fn remove(&mut self, obj_id: ObjId) {
        self.mobs.remove(obj_id);
        self.items.remove(obj_id);
        self.locations.remove(obj_id);
        // self.rooms.remove(obj_id);
        // self.spanws.remove(obj_id);
        self.equips.remove(obj_id);
        self.objects.remove(obj_id);
        self.labels.remove(obj_id);
        self.pos.remove(obj_id);
        self.vendors.remove(obj_id);
        self.prices.remove(obj_id);
    }

    pub fn get_mob_ctx(&self, mob_id: MobId) -> Option<MobCtx> {
        let mob = self.mobs.get(mob_id)?;
        let room_id = self.locations.get(mob.id)?;
        let room = self.rooms.get(room_id)?;

        Some(MobCtx { mob, room })
    }

    pub fn get_player_ctx(&self, player_id: PlayerId) -> Option<PlayerCtx> {
        let player = self.players.get(player_id);
        let mob = self.mobs.get(player.mob_id)?;
        let room_id = self.locations.get(mob.id)?;
        let room = self.rooms.get(room_id)?;

        Some(PlayerCtx { player, mob, room })
    }

    pub fn tick(&mut self, outputs: &mut dyn Outputs, delta_time: DeltaTime) {
        self.time.add(delta_time);

        if self.time.tick.as_u32() % 100 == 0 {
            debug!("tick {:?}", self.time);
        }

        let mut ctx = Ctx {
            container: self,
            outputs,
        };

        spawn::run(&mut ctx);
        mob::run_tick(&mut ctx);
        item::run_tick(&mut ctx);
        crafts_system::tick(&mut ctx);
    }

    //    pub fn save(&self, save: &mut dyn Save) {
    //        self.players.save(save);
    //        self.mobs.save(save);
    //        self.items.save(save);
    //    }
}
