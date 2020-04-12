# Main Todo

- command show map
- attach random generate map into loader
- configure spawns to trigger during start

# Design

# usability

- fix extra lines in output when look
- intelligent creation of codes for a label

# Space specific

- replace coord system by orbit by and orbit distance
- replace ship moviment by burn, travel, retroburn, cyclo burn etc
- secttormap
    - show numbers
    - show coords
    - allow to move by numbers
- time to land, time to launch
- see inside a Vehicle

# Fantasy specific

- casting
- equip
- armor
- experience
- improve combat 
  - dodge
  - aggressive
  - defensive
- potions

# Features

- TODO in config checker 
- serialization 
- crafting
- remove room_id from spawn (we could want ot have spawn per multiples rooms, zones? regions?)
- level up
    - separate XP that give on kill from accumulated
- add exit type
- add room size
- add put item at
- hire companions or controlled drones
- advanced parse commands like "examine drunk body" || examine body || examine body.2 || examine all body?
- persistence save and load
  - pre-requets to define a proper load and save format

# usability

- get usability by giving feedback with options when execute a get: get? get what? get what wher? get what in where?
- comm need to be defined by configuration files for localization
- there is no server buffering to send and receive messages, if user can not read, is possible that we lose output or stuck

# refactoring

- stop the infinite amount og mob.get during combat
- actions should not trigger instantatnius changes
- add time/tick to logs
- move outputs to Container, use container Buffer to remove the reference. Maybe not, we could just have events there
- normalize StrInput
- change static_id to strings
- re-organize code modules
  - better layering between view commands, game logic, container, etc.
  - simple channel of Commands in (including ticket) and outputs out. All parsing and serialization of messages need to 
    be done by controller layer
- split views, parsing, actions, domain methods and repository
- use ReadRepo<T> and WriteRepo<T>
- move room flags as children of room
- move Portal a children of room

