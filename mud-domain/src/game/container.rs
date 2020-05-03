use crate::errors::*;
use crate::game::astro_bodies::AstroBodies;
use crate::game::config::Config;
use crate::game::domain::{GameTime, MobCtx, PlayerCtx};
use crate::game::equip::Equips;
use crate::game::hire::Hires;
use crate::game::item::ItemRepository;
use crate::game::labels::Labels;
use crate::game::loader::Loader;
use crate::game::location::Locations;
use crate::game::memory::Memories;
use crate::game::mob::{MobId, MobRepository};
use crate::game::obj::Objects;
use crate::game::ownership::*;
use crate::game::player::PlayerRepository;
use crate::game::pos::PosRepo;
use crate::game::prices::Prices;
use crate::game::random_rooms::RandomRoomsRepository;
use crate::game::room::RoomRepository;
use crate::game::ships::Ships;
use crate::game::spawn::Spawns;
use crate::game::surfaces::Surfaces;
use crate::game::surfaces_object::SurfaceObjects;
use crate::game::system::{
    combat_system, item_system, rest_system, ship_system, spawn_system, SystemCtx,
};
use crate::game::tags::Tags;
use crate::game::timer::*;
use crate::game::triggers::*;
use crate::game::vendors::Vendors;
use crate::game::zone::Zones;
use crate::game::{item, mob, spawn, system, Outputs};
use commons::save::{Snapshot, SnapshotSupport};
use commons::{DeltaTime, ObjId, PlayerId};
use logs::*;

// #[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub astro_bodies: AstroBodies,
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
    pub memories: Memories,
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
            astro_bodies: AstroBodies::new(),
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
            memories: Memories::new(),
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
        self.astro_bodies.remove(obj_id);
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
}

impl SnapshotSupport for Container {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {
        self.config.save_snapshot(snapshot);
        self.time.save_snapshot(snapshot);
        self.objects.save_snapshot(snapshot);
        self.players.save_snapshot(snapshot);
        self.mobs.save_snapshot(snapshot);
        self.items.save_snapshot(snapshot);
        self.rooms.save_snapshot(snapshot);
        self.spawns.save_snapshot(snapshot);
        self.locations.save_snapshot(snapshot);
        self.equips.save_snapshot(snapshot);
        // self.tags.save_snapshot(snapshot);
        self.labels.save_snapshot(snapshot);
        // self.ships.save_snapshot(snapshot);
        // self.sectors.save_snapshot(snapshot);
        // self.astro_bodies.save_snapshot(snapshot);
        // self.pos.save_snapshot(snapshot);
        // self.surface_objects.save_snapshot(snapshot);
        // self.loader.save_snapshot(snapshot);
        self.vendors.save_snapshot(snapshot);
        self.prices.save_snapshot(snapshot);
        self.timer.save_snapshot(snapshot);
        self.triggers.save_snapshot(snapshot);
        self.ownership.save_snapshot(snapshot);
        self.zones.save_snapshot(snapshot);
        self.hires.save_snapshot(snapshot);
        // self.random_rooms.save_snapshot(snapshot);
        self.memories.save_snapshot(snapshot);
    }

    fn load_snapshot(&mut self, _snapshot: &mut Snapshot) {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn fields_into_save() {
        let fields = r"
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
        astro_bodies: AstroBodies::new(),
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
        random_rooms: RandomRoomsRepository::new()";

        for line in fields.split('\n') {
            let line: &str = line;
            let field = line.split(":").next().unwrap().trim();
            if field.is_empty() {
                continue;
            }

            println!("self.{}.save(snapshot);", field);
        }
    }
}
