# TODO

- player login and logut
- How to remove entity? from all components?
- fix spawn
- command inv
- fix extra lines in output when look
- move controller part from game into mud-server
- how we can introduce a new view that will stop to receive events? Like stunned?
- equipament affect stats 
- move portal to own index (to allow shipt to dock and connect)
- improve combat 
  - dodge
  - aggressive
  - defenseive
- seller and store
- serialization
- move to to mud-engine
- decouple messages (to allow other games/i18n) 
  - probably will not be necessary, even if we decide to re-use for space game

## Others

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

## long improvements

- comm need to be defined by configuration files
- there is no server buffering to send and receive messages, if user can not read, is possible that we lose output or stuck

## Refactoring

- add timers 
- add trigger
- use double index collection (Vec<Option<Secundaryid>, Vec<Component>)
- acceptance test do not need to use server
- normalize inventory in more generic way
- better layering between view commands, game logic, container, etc.
  - simple channel of Commands in (including ticket) and outputs out. All parsing and serialization of messages need to 
    be done by controller layer
- entity based, no Mob or Item, just Entity

# Design

## Modules

Repository
- update indexes and check for consistency
- do not generate outputs

Actions
- execute operations like pickup or attack
- generate output

Handlers
- parse and execute actions
- interact directly with the user

View
- control user session

## Separation

Input -> View -> Handler -> Action
                  
Handler - parse player input,
Action - execute the actions and publish events       

# Forum

## UUID vs Constants ID

Some entities like rooms and prefabs have strong require to be constants. We want to easy reference
from config files.

Normally mud are fully persistent word where, there is no default configuration and save. 

An alternatives:

1 enum { static_id, dynamic_id }
- very rusty
- can cause problem with by index collection and ecs

2 reserved range
- add like 1M prefix for any dynamic value
- if choose to low, we run out of ids
- if to high, a lot of overhead in index like structures

3. Reindex (WINNER)
- after load of default assets, all dynamic entities are loaded with a reindex 
  - this require 2 passes, one to generate all new indexes, one to complete references
