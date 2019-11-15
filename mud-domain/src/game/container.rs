use super::mob::*;
use super::player::*;
use super::room::*;
use super::spawn::*;
use super::domain::*;
use super::item::*;
use commons::{PlayerId, ObjId, DeltaTime};
use crate::game::obj::Objects;
use crate::game::location::Locations;
use crate::game::equip::{Equips};
use crate::game::{Outputs, spawn, mob, item};
use logs::*;
use crate::game::tags::Tags;
use crate::game::labels::Labels;
use crate::game::config::Config;
use crate::game::crafts::Crafts;
use crate::game::surfaces::Surfaces;
use crate::game::planets::Planets;
use crate::game::pos::PosRepo;
use crate::game::surfaces_object::SurfaceObjects;

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
    pub crafts: Crafts,
    pub sectors: Surfaces,
    pub planets: Planets,
    pub pos: PosRepo,
    pub surface_objects: SurfaceObjects,
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
            crafts: Crafts::new(),
            sectors: Surfaces::new(),
            planets: Planets::new(),
            pos: PosRepo::new(),
            surface_objects: SurfaceObjects::new(),
        }
    }

    pub fn remove(&mut self, obj_id: ObjId) {
        self.mobs.remove(obj_id);
        self.items.remove(obj_id);
        self.locations.remove(obj_id);
        // self.rooms.remove(obj_id);
        // self.spanws.remove(obj_id);
        self.equips.remove(obj_id);
        self.objects.remove(obj_id);
        self.labels.remove(obj_id);
    }

    // TODO: add Result or complete remove this method
    /// If mob have no room, a exception will be throw
    pub fn get_player_context(&self, player_id: PlayerId) -> PlayerCtx {
        let player = self.players.get(player_id);
        let mob = self.mobs.get(player.mob_id).unwrap();
        let room_id = self.locations.get(mob.id).unwrap();
        let room = self.rooms.get(room_id).unwrap();

        PlayerCtx {
            player,
            mob,
            room
        }
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
    }

//    pub fn save(&self, save: &mut dyn Save) {
//        self.players.save(save);
//        self.mobs.save(save);
//        self.items.save(save);
//    }
}

