# Todo

- show landed ship
  - support multiples enter portal
- launch  craft
- time to land, time to launch
- parametrize game initialization
- remove item prefab and just put it into limbo
- add put item at
- fix extra lines in output when look
- move controller part from game into mud-server
- how we can introduce a new view that will stop to receive events? Like stunned?
- equipament affect stats 
- move portal to own index (to allow ship to dock and connect)
- remove room_id from spawn (we could want ot have spawn per multiples rooms, zones? regions?)
- seller and store
- serializatio
- move to to mud-engine
- decouple messages (to allow other games/i18n) 
  - probably will not be necessary, even if we decide to re-use for space game

## usuability

- secttormap
    - show numbers
    - show coords
    - allow to move by numbers

## Features

- improve combat 
  - dodge
  - aggressive
  - defenseive
- weapon
- equip
- armor
- buy 
- sell
- experience
- level up
- potions
- multiples commands like: s e n f or sssee
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
- acceptance test do not need to use server
- normalize inventory in more generic way
- better layering between view commands, game logic, container, etc.
  - simple channel of Commands in (including ticket) and outputs out. All parsing and serialization of messages need to 
    be done by controller layer
- entity based, no Mob or Item, just Entity
