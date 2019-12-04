# Loader / Prefab / Init / Serialization

# Short term

- All id are u32
- Prefab get stored, objects get instantiated with same id

## Serialize static objects

To have better consistency, we should always serialize all objects, including static ones. All changes in static files need
to be executed as Migration.
- so what is the prupose of Prefab? Should we remove prefabs?
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
