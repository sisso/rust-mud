# TODO

- fly ship to places
- land ship

- parametrize game initialization
- remove item prefab and just put it into limbo
- add put item at
- fix extra lines in output when look
- move controller part from game into mud-server
- how we can introduce a new view that will stop to receive events? Like stunned?
- equipament affect stats 
- move portal to own index (to allow ship to dock and connect)
- remove room_id from spawn (we could want ot have spawn per multiples rooms, zones? regions?)
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

We can use generation, uuid or reserved spaces. Anyway we can store everything in a single u32, since we never sill use
so much. Anyways we can easly go to u64. 

As conclusion, any solution will require HashMap or special storages to hold Bidimensional vectors, or sparse vectors.

Easy to just use namespaces.

### Old


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

3. Reindex FAILED
- after load of default assets, all dynamic entities are loaded with a reindex 
  - this require 2 passes, one to generate all new indexes, one to complete references
- BUT NOT: we dont want to to know what values in the JSON are external references and what are just numeric

- but references in the end need to be ObjId, the instantiation of ObjId need to have reference to a globla mappaing 
  object, still, very painfull.


4. UUID

- solve all problems
- what differ from reserved range? 100000 reserved, dynamic
enum Id {
    Reserved { id },
    Uuid { str: String },
    Auto { id },
}

impl Id {
    pub fn as_u32 -> u32 {
        match self {
            REserved => self.0 + 100000
        }
    }
}

Id (id: generation)
- Bidimenstional storage
- 


## Optional room id

Most of commands always require a room_id. But make sense to have a room id.

ECS system solve it by just join between collections and ignoring the cased.

For simulate I need to manually flat every time we need it.

To facilitate we can just throw exceptions.
- no player command should happens if have no location
- so player context is expected to have a avatar, avatar should be in a place.

## Tag

4. HashSet? vector? Or struct?

3. Tags should be static or dynamic? We want to add logic, too dynamic means that we need to pass global containers
   to indicate what tags have what beahviours. No sense to keep this dynamic, if a module dont want  ,just don use it.
   
So it should be a enum, a custom value can be used for externam moduiles


1. Tags can be impelmented a specific service where we create tags

Tag CanBePickUp
Tag CanHoldItems

And we can set items tags

.set(chets, Tag.CanBePickUp);

2. Or tags can be just generic ObjId and we use location to define objects tags

let can_dock = objects.create();

locations.set(chest_id, can_dock);

- This solution will not work because same object can not belong to multiples locations


## prefabs

Since most of entities pieces get spread between components, the prefabs can be used as a container to hold all
the information together. 

Item Prefab { "armor", rd:2, 3gk}

better that

Item Prefab [ {label: armor}, {item.armor.rd: 2, ],etc

Or maybe not?

Maybe items prefab can be just builders, used for both test, initial. 

How to clone a full entity?

We can use serialization model to defined prefabs and entites.

## Serialization format 

1. Use what we current define as components
- easy
- dependencies between serialization and internal model

2. Use specific format
- manual mapping
- same format can be used to create stuff and save
- less dependencies between fromats


{
    headers: {
    }
    
    objects: {
        1023: {
        }
    }

    objects: [
        { 
            id: $id,
            labels {
            }
            {rooms
            
        }
    ]
}

## Ship and Crafts

- are a collection rooms, or a zone
- the out is a portal that connect both zones, from ship entrance to the landing pad
- ships can be in sector
- ship have position
- ship can be in planet
- some room in planet has the landpad


{ id: 10, spacecraft }

{ room: 1, zone: 10, label: airlock, exits: [(u, 2)] }
{ room: 2, zone: 10, label: airlock, exits: [(d, 1)], tag: [airlock] }

{ zone: 11, label: planet }
{ room: 3, zone: 11, label: landing pod, exits[], tag: [landpad] }

on land

zone 10, find airlock
zone 11, find landpad

create out form airlock to enter in landpad

{ room: 3, zone: 11, label: landing pod, exits[enter->2], tag: [landpad] }

airlocks always connect as out,

multiples entrances our exits.

A room with enter connection, can connect into a room with out connect
A room with enter connection can connect into multioples rooms with out connect

enter a
out

{
    id: 10,
    label: {
        label: "ship 1",
        code: ["label", "ship"],
    },
    spacecraft: {
        speed: 1
        cargo: 1000
    },
    zone {
    }
}

{ 
    id: 1,
    label: {
        label: "Airlock"
    },
    room: {
        zone_id: 10
        exits: [
            "u", 2
            "out", 3
        ]
    }
}

{ id: 11, label: { "landingpath" }, zone: {} }

{
    id: 3
    label: {
        label: "Landing pad"
    },
    room: {
        zone_id: 11,
        exits: [
            "enter", 2
            "enter", 23
        ]
    }
}

{
    id: 23
    spacecraft: {
    }
}

## Sectors and planets

Sector 1
- Planet 1 
  - Main room
  - Bar 
  - florest

- Planet 2
  -rocks

