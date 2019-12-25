# Main Todo

- add time/tick to logs
- continue time loader implemented
- add invisible items. Currently is possible to see a Wolf Spawn. Logical objects are Label or Obj? 

# Design

- how we can introduce a new view that will stop to receive events? Like stunned?

# usability

- fix extra lines in output when look
- intelligent creation of codes for a label

# Space specific

- secttormap
    - show numbers
    - show coords
    - allow to move by numbers
- time to land, time to launch
- see inside a Vehicle

# Fantasy specific

- casting
- weapon
- equipment affect stats 
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
- warning all non used properties in load files, just too many cases where something was not implemented and require debug
- serialization 
- seller and store
    - buy 
    - sell
- crafting
- remove room_id from spawn (we could want ot have spawn per multiples rooms, zones? regions?)
- level up
- add exit type
- add put item at
- hire companions or controlled drones
- advanced parse commands like "examine drunk body" || examine body || examine body.2 || examine all body?
- persistence save and load
  - pre-requists to define a proper load and save format
- surface in planets

# usability

- get usability by giving feedback with options when execute a get: get? get what? get what wher? get what in where?
- comm need to be defined by configuration files for localization
- there is no server buffering to send and receive messages, if user can not read, is possible that we lose output or stuck

# Refactoring

- change static_id to strings
- re-organize code modules
  - better layering between view commands, game logic, container, etc.
  - simple channel of Commands in (including ticket) and outputs out. All parsing and serialization of messages need to 
    be done by controller layer
- split views, parsing, actions, domain methods and repository
- use ReadRepo<T> and WriteRepo<T>
- add timers 
- add trigger
- move room flags as children of room
- move Portal a children of room
- use double index collection (Vec<Option<Secundaryid>, Vec<Component>)

