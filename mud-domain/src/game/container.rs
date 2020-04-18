use crate::game::astro_bodies::AstroBodies;
use crate::game::config::Config;
use crate::game::ships::Ships;
use crate::game::domain::{GameTime, MobCtx, PlayerCtx};
use crate::game::equip::Equips;
use crate::game::item::ItemRepository;
use crate::game::labels::Labels;
use crate::game::loader::Loader;
use crate::game::location::Locations;
use crate::game::mob::{MobId, MobRepository};
use crate::game::obj::Objects;
use crate::game::player::PlayerRepository;
use crate::game::ownership::*;
use crate::game::pos::PosRepo;
use crate::game::prices::Prices;
use crate::game::room::RoomRepository;
use crate::game::spawn::Spawns;
use crate::game::surfaces::Surfaces;
use crate::game::surfaces_object::SurfaceObjects;
use crate::game::tags::Tags;
use crate::game::vendors::Vendors;
use crate::game::{item, mob, spawn, Outputs, system};
use crate::game::timer::*;
use crate::game::triggers::*;
use commons::{DeltaTime, ObjId, PlayerId};
use logs::*;
use crate::errors::*;
use crate::game::system::{SystemCtx, ship_system, spawn_system, combat_system, rest_system, item_system};
use crate::game::zone::Zones;
use crate::game::hire::Hires;
use crate::game::random_rooms::RandomRoomsRepository;

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
    pub ships: Ships,
    pub sectors: Surfaces,
    pub space_body: AstroBodies,
    pub pos: PosRepo,
    pub surface_objects: SurfaceObjects,
    pub loader: Loader,
    pub vendors: Vendors,
    pub prices: Prices,
    pub timer: Timer,
    pub triggers: Triggers,
    pub ownership: Ownerships,
    pub zones: Zones,
    pub hires: Hires,
    pub random_rooms: RandomRoomsRepository,
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
            ships: Ships::new(),
            sectors: Surfaces::new(),
            space_body: AstroBodies::new(),
            pos: PosRepo::new(),
            surface_objects: SurfaceObjects::new(),
            loader: Loader::new(),
            vendors: Vendors::new(),
            prices: Prices::new(),
            timer: Timer::new(),
            triggers: Triggers::new(),
            ownership: Ownerships::new(),
            zones: Zones::new(),
            hires: Hires::new(),
            random_rooms: RandomRoomsRepository::new(),
        }
    }

    // TODO: use macro trait or some more generic way to remove all references
    pub fn remove(&mut self, obj_id: ObjId) {
        self.objects.remove(obj_id);
        self.mobs.remove(obj_id);
        self.items.remove(obj_id);
        self.locations.remove(obj_id);
        // self.rooms.remove(obj_id);
        // self.spanws.remove(obj_id);
        self.equips.remove(obj_id);
        self.labels.remove(obj_id);
        self.vendors.remove(obj_id);
        self.prices.remove(obj_id);
        self.ownership.remove_owner(obj_id);
        self.space_body.remove(obj_id);
    }

    pub fn get_mob_ctx(&self, mob_id: MobId) -> Option<MobCtx> {
        let mob = self.mobs.get(mob_id)?;
        let room_id = self.locations.get(mob.id)?;
        let room = self.rooms.get(room_id)?;

        Some(MobCtx { mob, room })
    }

    pub fn get_player_ctx(&self, player_id: PlayerId) -> Option<PlayerCtx> {
        let player = self.players.get(player_id)?;
        let mob = self.mobs.get(player.mob_id)?;
        let room_id = self.locations.get(mob.id)?;
        let room = self.rooms.get(room_id)?;

        Some(PlayerCtx { player, mob, room })
    }

    //    pub fn save(&self, save: &mut dyn Save) {
    //        self.players.save(save);
    //        self.mobs.save(save);
    //        self.items.save(save);
    //    }
}

