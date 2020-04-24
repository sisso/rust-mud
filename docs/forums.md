# CSV

Csv is much easy to manage multiple entries, add new fields and you can even keep references of static id like HOCON. 

A CSV <=> To json is a easy 2 ways conversion, while HOCON is always a single direction.

## Prefabs

Today we keep in a different array objects that are instantiate in start from those are prefabs. Same aproach need
to happens for CSV.



# Loading and Persistent Again

Is already defined:

- init is basically a load game for a pre-defined save game with extra initializer, like random stuff generator
- init files are used to create save game
- anything that need to be reference between save initializations, for instances, rooms, prefabs and configuration, can have
  a StaticId. StaticId will be used by migration tools and loader to convert into save files.
- StaticId can be a string or number

## Caso of uses

### Init game

    {
        cfg {
            initial_room: "room_initial" 
        }

        objects {
            room_initial {
                static_id: "room_initial"
                label: "Initial room"
                exits: [ dir: "n", id: ${objects.arean.static_id} }
            }
            
            arena {
                exits: [ dir: "n", id: ${objects.room_initial.static_id} }
            }
        }
    }
    
Considering the plans to convert all init data into csv. Should we do it?

# Random dungeons 

- a pre-defined room and direction, can spawn a full random zone like dungeon of forest. Sam can be applied for searching.

- these zones are temporary and can be re-randomize or removed in case of no player is active there

  zone_1_dungeon_entrance: {
    id: 9,
    label: "Dungeon entrance"
    desc: ""
    room: {
      exits: [
        {dir: "n", to: ${objects.zone_0_forest.id} }
      ]
    }
    parent: ${objects.zone_1.id}
  }
  
  zone_1_dungeon_zone: {
    id: 10
    random_zone {
        entrance {
            id: ${objects.zone_0.forest.id}
            dir: "s"
        }
        
        size: [5,5]
        
        num_spawn_points: [3,5]
        
        spawn_points: [
            {
                prob: 1.0
                min_distance_entrance: 2
                space_distance: 2
                max_spawns: 2
                spawn {
                  prefab_id: [{prefabs.skeleton.id}, {prefabs.sekeleton_sword.id}]
                  max: 4
                  time_min: 10
                  time_max: 60
                }
            }
            {
                prob: 1.0
                min_distance_entrance: 4
                space_distance: 4
                max_spawns: 1
                spawn {
                  prefab_id: [{prefabs.spider_small.id}]
                  max: 8
                  time_min: 10
                  time_max: 60
                }
            }
        ]
    }
  }

This will generate a random map. We need to populate spawn points. Each spawn can be added with some min/max distance 
from entrance, and a radius of affect.

With the previous defined config. A new zone will be created with 5x5 dimension. The entrance will be through the
south of forest in zone 0.

Now it is time to create spanws.

It will sort a number between 3 or 5 spawns to be created.

It will sort one of spawns giving the prob.

Will search a place far away of `min_distance_entrance` from the entrance of the random zone and distance enough of
of any other spawn in distance of `max(this.space_distance, i.space_distance)`

# CSV vs HOCON

List of CSV files that get converted into json looks much simpler to create/parse/manipulate over HOCON files. 

- how this improve ID? 
    - maybe using static_id in the scope of csv file
    
The file format is {kind}-{namespace}.csv

First column is {id} string

spawn-zone0.csv

    id, location_id, prefab_id,min_time,max_time,max
    wolf_0,rooms-zone0.forest,mobs-animals.wolf,30,180,4

# Space parent and orbit

We already have parent child relation, why do we need a new for orbiting?

Each AstroBody should depend of its parent to define its orbit, only a orbit distance value will exist. And in this
case will be zero for sum

# Modules support

Would be nice to organize bigger concepts into specific modules like Space, Fantasy, Room, SpaceShip, etc.

A new module require

- components and a repository
- input handling
- new events and triggers
- output handling

# Timers and Ticks

There are 2 type of timers. 

1) Timers: Are timers that need to happens in a specific point in feature, should not be postone

2) Ticks: Use delta time from last tick 

# Spawn with trigger

Check for trigger, then check for constraints, if succeed create the stuff, in any case, add to timer a new schedule

- how to deal with initial state? 
  - All spawn need to be initialize and create the default timer. 
  - If load a game, the timer will persist the timers
    - if a new spawn was added between reloads?
  - what to do if someone create a new spawn?
  - anyway I need maintenance task to check if a spawn was already initialize, how at least all spawns should have
    at least one schedule active
    
A central check every tick/time will still be require. Even if the spawn by scheduler, a 
check will still be require to confirm all spawns have timers.

Would be possible to have a trigger every time a new spawn is created? Both in bootstrap and dynamically? Spawn 
repository need to store all new entries that can be take.


# Event format

It is ququire to have a at least two collections to represent a Trigger. 

impl Trigger {
    fn schedule(event: Event);
    fn query(kind: Kind) -> Vec<Event>; 
}


Since rust dont give TypeId for Enum, it is not possible to use static code to map the type. 

The possible representations are:

a) 

const DecayKind: Kind = 0;
struct Decay {
    obj_id: u32,
}

b) 

enum Kind { Decay };
struct Decay { obj_id: u32  }

c)

enum EventKind { Decay };
enum Events {
    Decay { obj_id: u32 },
}

If mostly of cases Event will be just a obj_id, why I can not just use Event { kind: Kind, obj_id }



# Game receive independent commands v Input -> Output

A common flow simplify how things works. For instance, you don't need one advanced listener/trigger if we know that all messages are always processed later.

To create a commons flow, we should back to previous implementation where tick is Input -> Output. In previous impl Input contains new connections, inputs, time elapsed, everything. Output contains diconnects, commands and outputs.

Some operations like receive inputs and collect outputs are more user friendly by have direct commands, like connect()

Game can have some utilities mehtods, but controler, servvices and game loop should repsect the flow.

System should only run if there is a time updade.

## Multithread

Does this method will be still useful if we do multithread?

Yes, it will create a central point of control that will know how to split into parallel tasks. Anyway, receive random input anytime during gaming process will require locking

## Sequence

- disconnects
- connections
- inputs
- pre-system
- systems
- pos-system
- outputs

## Actions

Current actions are trigger direct from input like Pickup or Move. 

Ideally we should brake it int 2 steps, parsing and scheduling the action, a second flow to run the action.

Considering that any task will require time, everything will need to run into system anyways. 
- Need to be easy to schedule actions 

# Test layres

Containers, allow to test actions

Game allow to test commands, login, loader, etc

What layer should be used to test containers + serivces? 

Looks like is missing the right abstraction to contain all game logic.

Game
- controller
- services
- repository

# Trigger refactoring

Current model you can have multiples listeners per event. It is a flexible implementation but it require that all 
listeners need to keep handlers to access.

In our use case we are just interested to receive all trigger messages since last tick, this means that a simple
buffer would be enough. Everything else is just over engineering.

Anyways, it is ok, or even expected, that each system would have some state at some point. Conver the plain functions into a struct that hold state, cache or temporary state that don't depend of any entity was expected.

## How can we have a non handle trigger?

How to use trigger without require a state. 

We can just list all avaialble events of a kind.

How to cliean events?
- as normal algorithm, clean all already read events, if not listener, clean everything

To work the game loop should be:

inputs
process
systems
process triggers
clean up

a) depending of sequence of systems, one system can generate a trigger that need to consumed by previous system. Automatic clean up  do not work.

The easy solution is just have listeners everywhere

## Event kind in listenrs

As listener I can manage many different listeners, normally, I need to know what listener belog to each kind, so maybe make more sense to just have attached to the listner


# Modules

controller
- input and output
game
- model and repositories
logic

## Why

Things like timer and trigger will only work well if some structed is well defined. Currently send_input is triggering
direct changes into the game model.

## Notes

- logic modules a independent code or traits added into that receive continaer or specific services in arguments

## Flow

- read incoming messages from server
- controller process incoming messages into inputs or outputs
- game process inputs
- game process tick
- game process timer
- game process triggers
- controller.generated outputs from triggers
- send outputs to server

## How to achieve it:

- split logic from repository and moving to new module
- create in messages, inputs, outputs and triggers

## It was before?

This was exactly what we have before. The idea was break the big method into a small and more concise functions.

We can keep the same by just buffering events until tick is run

## Only actions

What about look? Examine? Enter <list candidates?>


# Tags

Probably the best way to handle cases like Vendor sell and buy. Even better that using a category.

Tags are numeric, the "string" are managed by just configuration files in custom conf file.

This will not be a issue when dealing with Modules? Is not exactly the same situation of StaticId in small scale?

# Spawn ownership

Currently to force spawn to lose ownership is require to destroy the object. It work good for mobs but not for 
items. If player has a item, it can happens to be merged and the old item deleted. 

it can be solved by just checking if the spawn area still contain the object with same prefab_id, or using ownership
model.
- looks like a good solution in mostly of cases, but since it can be solved by ownership that will need to happens
  at some time, is lost effort.
  - not if ownership show not useful
    - a door need a key, a key you need to have
    - control drone I need a key?
    - control orc I need a key? or loyality?

# Group money

Until a merge object is implemented, how to with collecting gold? 
- It could receive a especial treatment exactly how we are already dealing in vendor sell

# Merge objects

How do we deal with situation of merge objects? Like 10 gold coins, 2 swords,
1000 tons of Iron?

Can it be gui only?
- take iron.10 from 1000 iron is insane?
- can cause performance overhead

What should be attribute used to group it?
- label, or code?
  - since code is a array, we need to define priority and use the first one. Could work for ["sword", "broad"] and ["sword", "long"], but not for ["key","transport"], ["key", "house"].  Honestly, none of then look good.
    - only if use all codes
  - label could be used, but this mean that how I can distinguish two label if they code are the same? bringing back that it should be code.
 
Sometime dont't make sense to merge objects that re equals, same code, same label and same original prefab, it can contain different attributes like "use", "num of kills" or magic.
- real grouping should only happens for identical objects
  - not easy to solve in current multi component system

# Bussines logic

Currently mostly of logic is spread throuh handle inputs, execute commands and extra other operations. Put extra 
functions in the package is not a good idea because we want to reduce dependency from Repositories and Container. 

The idea is to split Component/Repository from bussines logic. This mean we need

1) create a new _bussines classes lilke

mob_service 
mob

2) move current model and repository into a new class and use the main one for business

mob
mob_repository

The option 2) will require more work, but adapt easy to logic that don't belong to any component or repository.

# Ctx or merge Outputs into container?

c) 

Considering that only few places are using Ctx (systems). Keep it or remove it is the lower effort to change for
now and if we decide otherwise. 

b)

Still CTX make sense, only with CTx we can move dyn references around without hardcode into container. Considering we
want to move container to hold just Repository. How will keep all references for services? If we decide to 
create a service holder?

a)

increase usage of CTX, or maybe just merge? In the long run outputs will be replaced 
by events. Events will belong to container. The Event -> string will be implemented
in another location. There is no point to ind

# Should handler receive target or full argument?

If I want to send full arguments I would use just the main handle. If I decide to use a input handly directly is because
I already know what I want, I don't want to keep repeating.

# Long naming

Some entities can have long names like "John der Kleiner' or "Light Transport". Normally each entity will have a list of
code that can be used to reference. 

One idea was to accept long names when issue commands, like "pick john head from small pot". This mean that  mostly of
command needs to deal with some advanced parser instead accept just a Vec<&str>

Probably will just easy to wrap the input in some new type like Input.

    struct <'a> Input(pub &'a str)

    impl Input {
    }

    input.has_command(input, "enter") -> bool
    input.has_commands(input, &["enter", "e"]) -> bool
    let say_msg = input.plain_arguments()
    let list_ids = input.argumetns_pieces()
    let (from, to) = input.parse_two_arguments("from")
    

Required functions to operate with long naming:



# Internal Items, Invisible objects

b) 

At current situation, is not about be invisible or internal. Spawn just should never be visible. The same would
be apply for different objects. Some are always visible (mobs, items, furniture, etc) and some are always invisible 
(spawn, other rooms, doors, etc), or better, is not about be visible, is non existent for player.

This mean that will be easy to check case by case. Have a function that define if a entity should be visible? can be
pick up? can be fight? etc.

a) 

Where it should be define? Global as object or in label?

The situations where internal objects need to be filter are? Always??
- we should never have any method that list/give to user whatever is there, we always should check what we try to show.
- this means that look/examine/get/put/etc must always know what type of object is manipulating. 
  - Get/Put always operate on Item
  - Look? Mobs, Items, Carft,???
    - not very extensible. every time we need to add a new thing, look and examine need to be changed.
    
What if Label only contains label for look and desc for examine?
- ok, but we need some label to track objects. We can use code, but code is designed to user interaction.
- only objects with description can be describe?
  - ok, but a bit confuse
- if we are considering to use label to define what can be seeing. Make sense to just flag in label that this
  object is internal, can not be used with look or examine
- still don't look good :/



# Item type

How very specifc items like GOLD?

This type of items require internal logic in many different parts of the systems, so they require to be referenced by
config files or have code flag.

Options to add int he code:

a) Boolean flags

Simple but verbose

can have issues to enforce mutual exclusivity.
- not a really issue, since all situations we have to choose, mutual exclusivity was always abandoned

b) Tags

It is basically same as A, but a more flexible and less optimized 

c) ItemKind

Dismissed since require mutual exclusivity

Conclusion:

Use flags and then move to Tags 

# Loot 

Loot is Items children that spawn 

To add random loot, we need to support some random children during spawn. 

The more flexible spawn system would be a probability tree. For instance:

    loot {
        2: {
          50: { item_id: [${reare_item_1_id}] }
          50: { item_id: [${rare_item_2_id}] }
        }
        
        28: {
          50: { item_id: [${uncommun_item_1_id}] }
          50: { item_id: [${uncommun_item_2_id}] }
         }
        
        70: {
          100: { item_id: [${common_item_1_id}, ${common_item_2_id}] }
         }
    }

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

## Full trigger model

Each input get added is a trigger. Each input convert into a command by a listener. Each command trigger actions. 
Each action trigger stuff (move room, create body if kill). Each stuff trigger a output. Each output is write to 
players.


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

## Tags

This same issue don't apply to tags? And looks like is good there 

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

Lets just use generic: String

Argument - you send wrong stuff
Invalid  - something is not what should be, maybe just using the wrong argument
Warning  - something is not what was expected to be, something is wrong and need to be verify
Error    - something is failing miserably

For each of those categories, when require, we could create custom exceptions to be catch.

ParserArgument { cause: ParseArgumentCause } 

---

In general we really don't care about the error, we just want to easy return from the block and the caller decide 
if want to deal or not with the issue.

Sometimes we generate warn logs

Sometimes we use to give different messages

---

Categories

complete failures, like fail to write file
unexpected failures, like a expected reference is invalid
illegal argument, you provide illegal argument


|command                        | type|
|-------------------------------|-----|
|fail to write save game        | IOError    |
|expected configuration or id don't exists | NotFoundError |
|'get' without argument                    | InvalidArgument|
|'get xxx' where xxx dont exists                       | NotFoundArgument |
|'get item', item was found but Item type don't exists | InvalidState |
|'get item', but item is stuck | InvalidArgument |
|'get item', but character is in limbo| InvalidState |
| can not attack because is resting ||
| item can not be used as weapon ||


----

As first implementation is already showing, a centralized Error can get chaotic very shortly. A clear structure needto be defined, that can be easily recognize and only hold useful values. 

Most errors we want to propagate and exploded with meaningful error message. This case we will just use Generic. 
- and how it improves by simple unwrap? At least panic contains a full stack trace.

General policy:
- panic for unexpected errors
- any other case that can expected to have a success/failure we return a error code
- error codes need to be obvious, a Conflict(u32) moved 3 layers out where it was produced will not bring any useful
  information. If objects insert will return a Conflict, it will need to be map to Generic(String) to be propagated

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
  - forest

- Planet 2
  -rocks

