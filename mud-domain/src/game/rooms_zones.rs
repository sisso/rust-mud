use crate::game::container::Container;
use crate::game::location::LocationId;
use crate::game::room::Room;

pub fn search_rooms_at(container: &Container, location_id: LocationId) -> Vec<&Room> {
   fn search_at<'a, 'b>(container: &'a Container, location_id: LocationId, buffer: &'b mut Vec<&'a Room>) {
      for (room, zone) in container.locations.list_at(location_id)
          .map(|obj_id| {
             (container.rooms.get(obj_id), container.zones.get(obj_id))
          })
          .filter(|(room, zone)| room.is_some() || zone.is_some())
      {
          if let Some(room) = room {
              buffer.push(room);
          }

          if let Some(zone) = zone {
              search_at(container, zone.id, buffer);
          }
      }
   }

   let mut buffer = vec![];
   search_at(container, location_id, &mut buffer);
   buffer
}