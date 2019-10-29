# General

## Disarm in Circle mud

As example how a code could be much simple https://www.circlemud.org/pub/CircleMUD/contrib/snippets/skills/disarm.txt

The following code all the require to a AI pick up a lost weapon after player cause it to disarm.

    if (LOST_WEAPON(ch)) {
      if (IN_ROOM(LOST_WEAPON(ch)) == IN_ROOM(ch)) {
        if (perform_get_from_room(ch, LOST_WEAPON(ch)))
          do_wield(ch, OBJN(LOST_WEAPON(ch), ch), 0, 0);
      }
      LOST_WEAPON(ch) = NULL;
    }

The same code will be require using our model

    if let Some(item_id) = container.lost_weapons.has_lost(mob_id) {
        if container.items.get_room_id(item_id) == container.items.get_room_id(mob_id) {
            if (actions::items::pickup(container, mob_id, item_id)) {
                actions::items::wield(container, mob_id, item_id);
                container.lost_weapons.remove(mob_id, item_id);
            }
        }
    }

# Old Mud

## Reusable actions 

Currently all inputs are forward to actions that parse input, forward to internal command and map into outputs. For instance
equip(player, arguments), forward to do_equip
do_equip receive again (player, arguments) -> Result
equip then match the result and output messages.

most of the code is just searching, then ask the repository to execute the real actions.

This impl have the following issues:

- very inflexible, can only be used by players doing inputs
    - only for player_id
    - receive <args> instead a well defined arguments
- the actions itself cannot be reused too, since will not generate output messages.

Solution

All inputs should be forward to a parser with the user, and arguments. The parser should then forward to a action with
the avatar mob, exactly require arguments. The action will be responsible to apply the change and generate the outputs.
- require what? like item_id? how should do a search the search?
  - probably a different parser method, in normal input we chain both. Internally we could decide between one
    or other.

# ECS

## Indexes

AtContainer is one of components that need to be indexed for fast access for many 
commands like: list at room, list my inventory, look in chest.

The system responsible to change AtContainers need to keep the index up-date.

Example of systems:

ItemSystem 
- get, put, drop

MoveSystem
- n, s, e, w
