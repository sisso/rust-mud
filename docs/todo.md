# Bug

- persistency
    - room memory

# Main Todo

- after login or switch a view, should show a first message
- remove prefabs from files and convert into a normal object
    - remove child

# Features

- admin verify / insert should support multiple lines 
- specs
- memory for the map
- portal distance 
- movements points
- commanding
  - by "say". "say all follow me", "say mercenary.1 wait here"
- teams
- potions and food
- advanced spawn
    - multiple rooms (zone?, radious?)
    - spawned creatures spawn/walk through rooms
    - flag to never spawn if player is viewing
- TODO in config checker
- serialization 
- crafting
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

- fix extra lines in output when look
- intelligent creation of codes for a label
- get usability by giving feedback with options when execute a get: get? get what? get what wher? get what in where?
- there is no server buffering to send and receive messages, if user can not read, is possible that we lose output or stuck

# refactoring

- merge space and fantasy scenery test functionality
- replace dyn buffer to vector one. Better, move it to container as Events, and later events to messages!
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

