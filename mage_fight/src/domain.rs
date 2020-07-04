pub type Pos = [i32; 2];
pub type Dimension = [i32; 2];
pub type Attribute = i32;
pub type Id = u32;
pub type Mana = i32;
pub type ActionPoints = i32;
pub type RoundNum = i32;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    IllegalArgument(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Spell {
    pub id: Id,
    pub label: String,
    pub cost_mana: Mana,
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
    pub attributes: Attributes,
    pub round_actions: ActionPoints,
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

    pub fn parse(value: &str) -> Result<Dir> {
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
            attributes: Attributes {
                hp: 10,
                mana: 10,
                actions: 2,
            },
            round_actions: 0,
        };
        let enemy_mob = Mob {
            id: 0,
            pos: [2, 0],
            team: Team::Enemy,
            attributes: Attributes {
                hp: 10,
                mana: 10,
                actions: 2,
            },
            round_actions: 0,
        };

        Game {
            arena: Arena::new(5, 5),
            player_mob,
            enemy_mob,
            spells: vec![],
            round: -1,
        }
    }

    pub fn start_game(&mut self) {
        self.next_turn();
    }

    pub fn is_player_turn(&self) -> bool {
        self.player_mob.round_actions > 0
    }

    pub fn handle_player_command(&mut self, command: Command) -> Result<()> {
        if self.player_mob.round_actions > 0 {
            match command {
                Command::Move(dir) => {
                    self.player_mob.round_actions -= 1;
                    Ok(())
                }
                other => Err(Error::Generic(format!("invalid command"))),
            }
        } else {
            Err(Error::Generic(format!("Not player round")))
        }
    }

    pub fn handle_ai(&mut self) {
        self.enemy_mob.round_actions -= 1;
        if self.enemy_mob.round_actions <= 0 {
            self.next_turn()
        }
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
