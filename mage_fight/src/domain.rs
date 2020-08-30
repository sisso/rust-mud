pub type Pos = [i32; 2];
pub type Dimension = [i32; 2];
pub type Attribute = i32;
pub type Id = u32;
pub type Mana = i32;
pub type ActionPoints = i32;
pub type RoundNum = i32;
pub type Duration = i32;
pub type Speed = i32;
pub type Damage = i32;
pub type Color = u32;
pub type ActionCost = i32;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    InvalidState(String),
    IllegalArgument(String),
    NotImplemented(String),
}

impl From<&str> for Error {
    fn from(v: &str) -> Self {
        Error::Generic(v.to_string())
    }
}

pub type GResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Spell {
    pub id: Id,
    pub label: String,
    pub codes: Vec<String>,
    pub cost_mana: Mana,
    pub time_to_cast: ActionCost,
    pub target: SpellTarget,
    pub deliver: SpellDeliver,
    pub effect: SpellEffect,
}

#[derive(Debug, Clone, Copy)]
pub enum SpellTarget {
    Myself,
    Friendly,
    Enemy,
    Pos,
}

#[derive(Debug, Clone)]
pub enum SpellDeliver {
    Projectile { speed: Speed, text_id: char },

    Focal {},
}

#[derive(Debug, Clone)]
pub struct SpellEffect {
    damage: Option<Damage>,
    heal: Option<Damage>,
}

#[derive(Debug, Clone)]
pub struct View {
    pub label: String,
    pub desc: String,
    pub codes: Vec<String>,
    pub text_identifier: char,
    // pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub enum Team {
    Player,
    Enemy,
}

#[derive(Debug, Clone)]
pub struct Attributes {
    pub hp: Attribute,
    pub mana: Attribute,
    pub actions: ActionPoints,
}

#[derive(Debug, Clone)]
pub struct Mob {
    pub id: Id,
    pub pos: Pos,
    pub team: Team,
    pub is_player: bool,
    pub attributes: Attributes,
    pub round_actions: ActionPoints,
    pub view: View,
}

#[derive(Debug, Clone)]
pub struct Arena {
    pub size: Dimension,
}

impl Arena {
    pub fn new(size_x: i32, size_y: i32) -> Self {
        Arena {
            size: [size_x, size_y],
        }
    }

    pub fn is_valid(&self, pos: Pos) -> bool {
        pos[0] >= 0 && pos[1] >= 0 && pos[0] < self.size[0] && pos[1] < self.size[1]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum Dir {
    N,
    S,
    W,
    E,
}

impl Dir {
    pub fn inv(&self) -> Self {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Dir::N => "n",
            Dir::S => "s",
            Dir::E => "e",
            Dir::W => "w",
        }
    }

    pub fn parse(value: &str) -> GResult<Dir> {
        match value {
            "n" | "north" => Ok(Dir::N),
            "s" | "south" => Ok(Dir::S),
            "e" | "east" => Ok(Dir::E),
            "w" | "west" => Ok(Dir::W),
            _ => Err(Error::IllegalArgument(format!(
                "Invalid argument: {}",
                value
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Exit,
    Wait,
    Move(Dir),
    CastSelf { spell_id: Id },
    CastAtPos { spell_id: Id, pos: Pos },
    CastAtTarget { spell_id: Id, target_id: Id },
}

#[derive(Debug, Clone)]
pub enum Event {}

#[derive(Debug, Clone)]
pub struct Game {
    arena: Arena,
    player_mob: Mob,
    enemy_mob: Mob,
    spells: Vec<Spell>,
    round: RoundNum,
}

impl Game {
    pub fn new() -> Self {
        let player_mob = Mob {
            id: 0,
            pos: [2, 0],
            team: Team::Player,
            is_player: true,
            attributes: Attributes {
                hp: 10,
                mana: 10,
                actions: 2,
            },
            round_actions: 0,
            view: View {
                label: "player".to_string(),
                desc: "The Player".to_string(),
                codes: into_vec_string(vec!["player", "me", "self"]),
                text_identifier: '@',
            },
        };
        let enemy_mob = Mob {
            id: 1,
            pos: [2, 4],
            team: Team::Enemy,
            is_player: false,
            attributes: Attributes {
                hp: 10,
                mana: 10,
                actions: 2,
            },
            round_actions: 0,
            view: View {
                label: "enemy".to_string(),
                desc: "An enemy".to_string(),
                codes: into_vec_string(vec!["enemy", "other", "o"]),
                text_identifier: 'O',
            },
        };

        let spell_firebolt = Spell {
            id: 2,
            label: "Firebolt".to_string(),
            codes: into_vec_string(vec!["firebolt", "fbolt", "fb"]),
            cost_mana: 1,
            time_to_cast: 1,
            target: SpellTarget::Enemy,
            deliver: SpellDeliver::Projectile {
                speed: 4,
                text_id: '*',
            },
            effect: SpellEffect {
                damage: Some(3),
                heal: None,
            },
        };

        let spell_heal = Spell {
            id: 3,
            label: "Heal".to_string(),
            codes: into_vec_string(vec!["heal", "hl"]),
            cost_mana: 1,
            time_to_cast: 1,
            target: SpellTarget::Myself,
            deliver: SpellDeliver::Focal {},
            effect: SpellEffect {
                damage: None,
                heal: Some(5),
            },
        };

        let spells = vec![spell_firebolt, spell_heal];

        Game {
            arena: Arena::new(5, 5),
            player_mob,
            enemy_mob,
            spells: spells,
            round: -1,
        }
    }

    pub fn start_game(&mut self) {
        self.next_turn();
    }

    pub fn is_player_turn(&self) -> bool {
        self.player_mob.round_actions > 0
    }

    pub fn get_arena(&self) -> &Arena {
        &self.arena
    }

    pub fn handle_player_command(&mut self, command: Command) -> GResult<()> {
        if self.player_mob.round_actions > 0 {
            match command {
                Command::Move(dir) => {
                    do_mob_move(&mut self.player_mob, &self.arena, dir)?;

                    self.player_mob.round_actions -= 1;
                    Ok(())
                }
                other => Err(Error::NotImplemented(format!(
                    "Unexpected command [{:?}]",
                    other
                ))),
            }
        } else {
            Err(Error::InvalidState(format!("Not player round")))
        }
    }

    pub fn handle_ai(&mut self) {
        self.enemy_mob.round_actions -= 1;
        if self.enemy_mob.round_actions <= 0 {
            self.next_turn()
        }
    }

    pub fn get_mobs(&self) -> Vec<&Mob> {
        vec![&self.player_mob, &self.enemy_mob]
    }

    fn next_turn(&mut self) {
        // increase round
        self.round += 1;

        // refresh mobs action points
        self.player_mob.round_actions = self.player_mob.attributes.actions;
        self.enemy_mob.round_actions = self.enemy_mob.attributes.actions;
    }
}

// fn move_pos(pos: Pos, dir: Dir, dimension: Dimension) -> Option<Pos> {
//
// }
//
fn do_mob_move(mob: &mut Mob, arena: &Arena, dir: Dir) -> GResult<()> {
    let mut pos = mob.pos.clone();

    match dir {
        Dir::N => pos[1] -= 1,
        Dir::S => pos[1] += 1,
        Dir::W => pos[0] -= 1,
        Dir::E => pos[0] += 1,
    }

    if arena.is_valid(pos) {
        mob.pos = pos;
        Ok(())
    } else {
        Err("Invalid direction".into())
    }
}

fn into_vec_string(v: Vec<&str>) -> Vec<String> {
    v.into_iter().map(|i| i.to_string()).collect()
}
