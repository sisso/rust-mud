
# Todo

- project Err: IllegalArgument, Fatal, Error, Unknown, Warning, etc
- replace exists types Enter/Out by specific actions in airlock and landing pad
  - remove automatic portal creation when ship land or undock
- intelligent creation of codes for a label
- parse initial configuration files
- support multiples enter portal
- time to land, time to launch
- add exit type
- surface in planets
- move Portal a children of room
- move room flags as children of room
- parametrize game initialization
- remove item prefab and just put it into limbo
- add put item at
- fix extra lines in output when look
- move controller part from game into mud-server
- how we can introduce a new view that will stop to receive events? Like stunned?
- equipment affect stats 
- move portal to own index (to allow ship to dock and connect)
- remove room_id from spawn (we could want ot have spawn per multiples rooms, zones? regions?)
- seller and store
- serialization 
- move to to mud-engine

## usability

- secttormap
    - show numbers
    - show coords
    - allow to move by numbers

## Features

- improve combat 
  - dodge
  - aggressive
  - defensive
- weapon
- equip
- armor
- buy 
- sell
- experience
- level up
- potions
- hire companions
- advanced parse commands like "examine drunk body" || examine body || examine body.2 || examine all body?
- load predefined world from file
- persistence save and load
  - pre-requiste to define a proper load and save format

## improvements

- comm need to be defined by configuration files for localization
- there is no server buffering to send and receive messages, if user can not read, is possible that we lose output or stuck

## Refactoring

- use ReadRepo<T> and WriteRepo<T>
- send messages to mob_id instead player_id
- add timers 
- add trigger
- use double index collection (Vec<Option<Secundaryid>, Vec<Component>)
- better layering between view commands, game logic, container, etc.
  - simple channel of Commands in (including ticket) and outputs out. All parsing and serialization of messages need to 
    be done by controller layer
