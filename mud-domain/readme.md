# TODO

- noramlize ID
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

