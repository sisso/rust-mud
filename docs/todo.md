# Todo

- remove surface and reimplement space by using a list of celestial systems

# Design

- how we can introduce a new view that will stop to receive events? Like stunned?

# usability

- fix extra lines in output when look
- intelligent creation of codes for a label
- secttormap
    - show numbers
    - show coords
    - allow to move by numbers

# Features

- Timers
- Triggers
- time to land, time to launch
- serialization 
- seller and store
- see inside a Vehicle
- improve combat 
  - dodge
  - aggressive
  - defensive
- remove room_id from spawn (we could want ot have spawn per multiples rooms, zones? regions?)
- weapon
- equipment affect stats 
- equip
- armor
- buy 
- sell
- experience
- level up
- potions
- add exit type
- add put item at
- hire companions
- advanced parse commands like "examine drunk body" || examine body || examine body.2 || examine all body?
- load predefined world from file
- persistence save and load
  - pre-requiste to define a proper load and save format
- surface in planets

# improvements

- comm need to be defined by configuration files for localization
- there is no server buffering to send and receive messages, if user can not read, is possible that we lose output or stuck

# Refactoring

- send messages to mob_id instead player_id
- re-organize code modules
- split views, parsing, actions, domain methods and repository
- use ReadRepo<T> and WriteRepo<T>
- add timers 
- add trigger
- use double index collection (Vec<Option<Secundaryid>, Vec<Component>)
- better layering between view commands, game logic, container, etc.
  - simple channel of Commands in (including ticket) and outputs out. All parsing and serialization of messages need to 
    be done by controller layer
- move room flags as children of room
- move Portal a children of room
- rename land-pad and airlock
  - landpad: hangar? landingpad? port? etc??

