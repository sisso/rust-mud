# Timers

All non instantaneous tasks will require some timer to keep track. Currently almost every long
operation require a check every tick by nextTime < totalTime.

Event machine is easy. The main issues are:

1) Track of what need to execute when trigger

a) Store the closure to execute

b) Run a trigger

2) Serialize scheduled tasks


## Implementation

Each system can schedule tasks by (namespace_uuid) -> arguments.

namespace_uuid is used to cancel, check, notify the owner that timer has trigger. 

argument is used to forward variables.

namespace_uuid and all arguments need to be serialize. So the timer will restrict the types.

# Triggers

Triggers require that a event dispatcher and subscriber. 

Don't necessary means that they are callback. Triggers can be processed by a systems that iterate though all generated
events in normal execution.

In a ideal use case, the full architecture need to be model in a way that all messages become events. Triggers respond
to events. All player messages are basically a trigger listener.

## Sequential vs lazy triggers

A trigger system can process any event existent in same tick. The order of systems is very important to process all
executed.

A lazy trigger will process all events generated in last tick. This is usually more stable but impossibility many
uses cases like "send messages to users when something happens" without using a high fps.

# Ownership

Something like location, ownership define what owns what. This will be used to defined avatars, who owns what itens and
what entities can be controlled by other entities,

Owneship will define the spawn. And owneship of spawn can be transfer

# StaticId u32 vs String

Unique number are complicated to maintain for humans considering multiples modules, namespaces, files and matching 
between save games. 

While possible to use tools to generate max id it is require. We already have path in hocon that is exactly the 
static_id.

Numeric with tools make almost impossible to manage mods. Any numeric namespace like (mod_id * 1000 + id), create many
unnecessary limitations and remove any expected optimization using indexed arrays.

# Surface deprecation

In terms of MUD would be better to just a list of planets and movement between celestial objects by "injection burn" ->
"cycling burn" -> "orbital align" -> "dead transfer" ,etc.

Starmap 

$ sm
Sum
- Earth
  - YOU 
  - ISS (Station)
  - Luna (Moon)
    - Orbital (STATION)
    - Dragon (Cruiser)
- Cargo 1 (in flight to Luna)
- Venus

$ move Orbital
Calculating transfer.
burn cost 5.000 DV, new fuel is 15000 DV
Executing alignment burn.
Executing launch burn.
Launch complete, arrival in 38 seconds.
Computing re-orbiting
Executing cycling burn.
Transfer complete.

$ move sol earth luna orbital 200k 3
Calculating transfer.
burn cost 15.000 DV, new fuel is 5000 DV
Executing alignment burn.
Executing launch burn.
Launch complete, arrival in 12 seconds.
Computing re-orbiting at 200km
.
.
.
## Solar system structure

Negative                                               Positive
64 - 32 - 16 - 8 - 4 - 2 - 1 - 0 - 1 - 2 - 4 - 8 - 16 - 32 - 64
                              Sum
                               Mercury
                                 Venus
                         Earth
                       Moon
               Mars
                                            Netuno
       Urannu

## Model

In the end is like a space hierarchic, every object is in orbit of another object in space. While not true relevant, 
would be interesting to keep track.

SpaceObject

Planet {
}

# Game control connections?

In what scope Game should manage players?

ConnectionId, PlayerId or MobId.

ConnectionId: Is what represent a real, single user. It is the identifier for the input and output, and the key for 
a specific user state.

PlayerId: Represent a in game player, each player can have multiples connections and have non or more Mobs. A player_id
is attached to a connection_id through login.

MobId: It is mobs controlled by player. A player can have none or multiples assigned to him. A mob_id is attached to 
player_id after character creation.

Mostly game rules should be applied in MobId. This mean that any mob, player controlled or no, can interact with the 
world in a easy way.

The use of player_id would be used only for advanced things like:
- change avatar
- change player persistent configuration

## Posses, Connect to Matrix or VR a mech

These are all commands that will require change temporary or permanently the avatar of a player.

Player will need to keep a set of IDs:
- avatar: MobId - Represent the player in the world, the way to kill you. A human or the hive mind, it is your.
- monitor: Vec<MobId> - List of mobs that you activilly listen, all messages send to those mobs you will receive as 
  your own. Including commands failure like, you can not move N
- controlled: MobId - Is the mob_id that own any action you type

In general case, avatar, monitor and controlled are same id. A posses command will switch both monitor and controlled. A
camera system can add monitor. 

Maybe: Outputs will need to be extend to split private outputs between "controller only" and monitor

# Vendors

Vendors should contain a list of ID to sell, or should items have a tag that vendor can sell?
- each vendor will keep its own list. But the list can be centralized by reference files in hocon

# Error Codes

As first implementation is already showing, a centralized Error can get chaotic very shortly. A clear structure need
to be defined, that can be easily recognize and only hold useful values. 

Most errors we want to propagate and exploded with meaningful error message. This case we will just use Generic. 
- and how it improves by simple unwrap? At least panic contains a full stack trace.

General policy:
- panic for unexpected errors
- any other case that can expected to have a success/failure we return a error code
- error codes need to be obvious, a Conflict(u32) moved 3 layers out where it was produced will not bring any useful
  information. If objects.instert will return a Conflict, it will need to be map to Generic(String) to be propagated

# Craft vs Ship

Craft is the right term to define any space think, like a shuttle is a Craft

Craft is the right term to make things.

Ship even if wrong, will cause much less ambiguity. Vehicle could be a better and more generic 
option.


# Loader / Prefab / Init / Serialization

## Inheritance prefabs

Be able to create a hierarchical helps to modularize better and solve a lot of copy paste during data definition. Each 
data definition can extends another. 

The same behaviour can be used to make better save game where only non matching properties are stored, only what
really differ from base will be store.

This come back the requirement of have a common and flexible structure like JSON that can be easy manipulated without
knowledge of each value. Maps between init files, save games and internal model will require.

During start, init files are read and convert into JSON. Loading/spawn are based in the JSON format. During save, 
internal model is convert into JSON, and only what differs from original is saved.

This implies: 
- Each OBJ can have a StaticId a parent.
- Each StaticId can extends another StaticId

a) How to deal with composite spanws with multiples references and inheritance?

b) Data format was designed to be the serialized representation file, used after init, to spawn and load/save. 

## Spawn prefab VS apply prefab

The spawn process can be simplified and increase flexibility by change a spawn method, that create object, prepare 
components giving a prefab, by a apply prefab, that just create the required components by a prefab. 

With new method we split some responsibilities and would be possible to apply multiples prefabs to the same object. 

## IDS

We have 3 types of IDs. Static ID that are fixed between server instantiations. Prefab IDs that are used by spawn and instantiation. Dynamic ID that are normal entities created during the gameplay.

Some options

enum ObjId {
    Static(u32),
    Dynamic(u32),
    Prefab(u32)
}

impl Id {
  fn as_u64(&self) {
    match self {
      Static(value) => value as u64 + 100000,
      Prefab(value) => value as u64 + 200000,
      Dynamic(value) => value
  }
}

enum IdKind { Static, Dynamic, Prefab }
struct Id { kind: IdKind, num: u32 }

## Hocon vs JSON

Hocon have many advanced features, including variable reference that save issues of change ID and allow intelj help
with the object navigation.

JSON is simple and more flexibe, but much more verbose. 

Hocon complexity can be minimized by tools, like validate unique ids and double check that all references are always
reachables.

## Children / Parent

Parent is easy to extend and is more independent, you can just add new object and set the parent, without need to
apply any change to the parent object. However is only useful in for unique objects like rooms. For classic items, a 
single item prefab can be part of many other mobs prefabs. 

Use only children can cause a lot of extra work for situation like Zones.

Conclusion: Children is more generic, but parent is more useful for static data or cases where some parent have too
many unique children. 

So we will just support both. But each one with different semantic. Parent will be used by static objects while children for prefabs. When instantiate, all children elements will be created with new Id, if used in static data, means that everytime server reload, a new object will be created  

## Short term

- All id are u32
- Prefab get stored, objects get instantiated with same id

## Serialize static objects

To have better consistency, we should always serialize all objects, including static ones. All changes in static files need
to be executed as Migration.
- so what is the purpose of Prefab? Should we remove prefabs?
  - migrations can already only happens in prefab model?
  
Maybe instances of prefab can be stored as enum Some[T], Inherited or None

## Migration

At some point database will require migration. Like update attributes from prefab, kill a mobs of a type, update something else.

Init files need to be manually migrated. Save files need to contains a version that will cause the upgrade.

Probably in a model much like database update.

## Scenery spawn prefabs?

How do we create a mob in prefabs. And boot strap the mob?
If mob will spawn with prefab static id, what happens if it get killed and removed from DB. Next boot will re-spawn?

Looks like initialize is only valid for static data. 

## Prefab vs Cloning

The idea of prefab is a fat struct that contain all fields as options. Before instantiation, validation is realized
and all components created.

This free model is much more easy to manipulate from user interfaces, serialization, reads and writes.

For instance:

    !prefab create 5
    !prefab set 5 label "Short-sword"
    !prefab set 5 damage 2
    !spawn create target-prefab 5 min-time 10 max-time 100 max 1

Cloning can be useful. But the algorithm to search for components, clean up undesired state can be very error prone. Even
with cloning, will be still require a easy way to manipulate and serialize entities.

### Conclusion

Prefab is required, is more useful, is more flexible, is less error prone.

## Prefabs 3

Scenery need to have stable ID that will be still valid during persistence.

Is not require that prefabs have String ID
- we will use u32 since is used anywhere and more compatible
  - later we can always implement tooling
  
Annotate objects as prefab?

Initialize as second step?
- this could more interesting and flexible for testing and exchange between modules. In the situation that is require 
  to boot the server only with some regions, you need to remove other files or change some init objects as non default.
- considering that spawn root will spawn any child, spawn the city without the mobs? 
  - still will require a lot of manual configuration
- currently I just need to switch parent from objects to prefabs, if some entities need to get out, I just need to move those



## Item Prefab 2

All loader objects are prefabs. They need to be materialized to generate a ID. 

Any object that need to create another object from prefab, will always reference
the string key in load.

The loading process is create all Prefabs and instantiate the root? 
- no, we dont want to put everything always in a single tree
  - there is no root anymore

But we can have a bootstrap list. 
- no, because most of things will be in bootstrap

The better is to split files, prefabs and auto initilize.


## Loader models

Files are just a bunch of keys, each key is mapped to a static id.

Values are flatten independent of current model, but similar. For instance, 
we will have fields label and desc, that will mapped to Label { label:, desc }

Most of fields will have direct map like item, mob, room, craft, etc.

    {
        shuttle-1: {
            label: "Shuttle-1"
            desc: "Shuttle very nice"
            
            children: {
                bridge: {
                    label:
                    children: {
                        control: {
                            label: "panel control"
                            item: { craft_control: true } 
                        }
                    }
                }
            }
        }
    }

## Loader

Load a bunch of configuration files that can be easy merge. All IDs and 
references should be unique strings.

A string_id -> id mapping is generated automatically and keep persistent. 

The string_id -> id is used to create entities during the load and keep
references from dynamic objects.




## Item Prefab

Any component can be just moved into limbo to be used as prefab. It will 
not make things complete easy since some of components will contains 
references that need to be mapped when clone a entity.

A builder can be used both for testing, parsing and prefab generation. 
But how?

PrefabObject -> Builder -> build(contianer).

So a prefab is a ID to a function that create a object tree. This object
tree can be defined in configuration files.


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

# Others

## Normalize texts

How do we normalize outputs when we can have request entries like login:
- we just remove it, all outputs should finish in new lines, entries will always be after the prompt, or even
  line after the prompt itself
  
How define what will be the prompt?
- maybe a specific ActionController 
## Send Msg(targets:vec<connection>, msg: String)

Send messages for multiple clients only work in very small scale, most of times the final string will be always 
a custom message that include things like player $hp$ anyways.
  
Easy to just simplify and optimize later (as always)

## Body as item, or mob?

Feature: change body to a dead mob, it will be never possible to store all info into Item. Many things like resurrect will be broken.

Have a body as a mob will create difficulties like carrie a body? should we carry a mob?

In the end, is not about be a Mob or a Item, but both elements need to be able to be mixed. A item is a mob, a mob is item. You can carrier a mob, or you can kill a item. This bring us back to the component based discussion.

## File format and Serialization

Considering that we will need to have save and write anyways. Why not just use save format for definition? 

A game is just a collection of configuration files, init or load we just load default list of files, then merge, them read.

Hocon
- is a mergeable by default
- support variables and references

This is for easy definition, but it will need to be converted to the load format
to allow to start the game. In this way we have a indirection and a single parse mechanism

## Changes in persistent world

In the case we have a long running persistent. How can I add new areas? obs and objects?
- the way will be use the data files that will always be loaded before the persistent files.

How to deal with id conflict?

{id: 1, component: location, parent: 2 }
{id: 2, component: room, exits: [(N,3)]
{id: 3, component: room, exits: [(S,2)]

Magic mapping is not good becase we dont want to track in save files which ids are just numeric, and which ones
are references.



## DoAction and messages

Testing with do_pickup can improve a lot. But how to deal with comm messages? Most of time all Error enuns will need
to fetch the information again just to show the error. For instance.

PickError.ItemNotFoundIn { inventory_id }, we will need to fetch the inventory again just to show whats options
are available and why we can not found it.

## Login async?

The plan is to have all communication between engine as a single channel. 

This will create by default a system that will support multiples controllers/servers. Otherwise is to easy to simple decide to add a if and print messages.

What about login? We send a login message?

Engine is not aware of connection since we are stabilizing only during login. 

It will require UserId and PlayerId

This give the power to sudo or even same player connected multiple times. Same playerId, multiples userId.

A lot of random benefit, none required

SOLUTION: Lets keep for now and verify further

## View Controler Flow

- view need to
  - login

## ECS Container, At Room

We can use a generic Container to hold other objects. Any object that can contain other object ( like all ??? ) will
implement it.

How do we represent rooms, inventory and mobs? How do we restrict a player not pick up other? 
- Weight? Volume?
    - so we can easy pick up small enemies?
        - if you have skill, what the problem if a giant pick you up?
    - we can say only pick items
    
## 3D MAP


....... 1) x - You    z+2 
.x..... 2) y - Planet z-1
.......
....y..
.......

1) You    ( 2, 2,  2)
2) Planet ( 5, 4, -1)

....... 1) x - You    z+2 
.x..... 2) y - Planet z-1
.......
....y..
.......

1) You    ( 2, 2,  2)
2) Planet ( 5, 4, -1)


## Ship Aerotinamic

To land into atm places

0.1-0.8 - smoth shape
0.8-2.0 - wings (bad for very dense? -% fuel?)

Atm thrusters
+100% fuel

Aquatic thruster
?

## XML format

While lthe XML format is better that I would like to assume. Still assume that you can only have a "Kind". While when using a ECS system usually you will have many types. For instance, instead:

<surface name="Planet" type="2D" size="10">
</surface>

Would be more interesting to have

{
    id: 1
    label: "Planet"
    surface: {
        size: 10
        type: 2d
    }
    planet: {
        atm: 0.8
    }
    position: [0, 2, 3]
    at: sector1
    
}

## Surfaces

Instead sector, we will introduce a new model Surface, that can be 2D or 3D.

Sectors, planets, mons and astteroids will implement surfaces.  Where space and aquatic reagions have 3d space.

Childrens of a Surface need to have position

SurfaceBody define peroperits of a body in surface. Like can be landaded? 

How to split planet atmosefpher and gravity from a city in a planet?

I have sector, that have planet, that have surface, that have city, that have rooms

I have sector, that have space station/craft, that have room

As code is just

if  me.get_location.is_surfac {
    locations.list(me.get_lcation).zip(positions) { obj , pos {
    }
} else if m.location.is_room {
}

sector {
    sector1 {
        label: "Sector 1"
        surface: { 
            kind: "space", 
            dimensions: 3, 
            bounds: 10 
            objects: {
                dune: {
                    location: "${surfaces.sector1.id}"
                    label: "Dune",
                    position: [2,3,0]
                    planet: {
                        atm: 0.8
                    }

                    surface: {
                        kind: "desert",
                        objects: {
                            palace_city: {
                                pos: [1, 4],
                                rooms: {
                                    palace {}
                                    landing-pad {}
                                    city {}
                                }
                            }
                        }
                    }
                }
            }
            station-1: { 
                station: {}
                rooms {
                    docking {}
                    airlock {}
                    quarters {}
                    bridge {}
                    galley {}
                }
            }
        }
    }
}


{
    "1" {
        surface {
        }
    }
    
    "2" {
        location: 1
        planet {
        }
    }
}

### Parent Child ves parent_id

parent_id:
- is more verbose, require more infrastructure to parse and more attention to work
- more flexible to append new child from different files if we want to go into mod way

## Piloting

Many vehicles are planed to be available and allow player to move around.

Craft - to fly in space and land into planets
Vehicels - to drive/fly/dive into planets, moons, asteroids
Feets - to walk around rooms
Mechs - PErsonal Mech are wearing, Large mech are vehicles

## Planet vs SpaceBody

Sectors 
- Planet
  - city
   - room
   - craft
     - room
- Craft
  - room
  
Station in space has same semantic as City in a planet

We will just call places. Places are things in Surfaces.

## Msg to playerid

We never want to send message to players, we always want o send message to avatars. A player can monitor many avatars. 
For example, the person itself, a drone, the current vehicle and current ship.

A message to the ship will hit all people inside, a message to vehicle only for who is incide, for the character, only himself.

This means that we want to attach player_id, or better, connections, to receive any message to the attached resources.

But how do we solve the issue if the pilot send a wrong command? everybody receive a spawn?
- well, the ship can shake and all fell
- if ship was not affected, only the mob_id that execute the action will receive the message.

Conclusion: 
- each mob can be watched by one or more player_id, messages are never to player only to mobs. 
- e should process all messages and map into mob_id

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

