use crate::{
    pokemon::{Pokemon, Team},
    state::{ActivePokemon, DecisionBuilder, Node, State},
};

#[derive(Debug, Clone)]
pub struct SingleBattle {
    player_1: PlayerState,
    player_2: PlayerState,
}

#[derive(Debug, Clone)]
struct PlayerState {
    active_pokemon_idx: Option<usize>,
    team: Team,
}

#[derive(Debug, Clone)]
pub enum Player {
    Player1,
    Player2,
}

impl State for SingleBattle {
    type Player = Player;

    fn active_pokemon(&self) -> ActivePokemon<'_> {
        let mut active = ActivePokemon::new();

        if let Some(i) = self.player_1.active_pokemon_idx {
            active.push(&self.player_1.team[i]);
        }

        if let Some(i) = self.player_2.active_pokemon_idx {
            active.push(&self.player_2.team[i]);
        }

        active
    }
}

impl SingleBattle {
    pub fn start(player_1_team: Team, player_2_team: Team) -> Node<SingleBattle> {
        let state = SingleBattle {
            player_1: PlayerState {
                team: player_1_team,
                active_pokemon_idx: None,
            },
            player_2: PlayerState {
                team: player_2_team,
                active_pokemon_idx: None,
            },
        };

        choose_starting_pokemon(state)
    }
}

fn choose_starting_pokemon(state: SingleBattle) -> Node<SingleBattle> {
    DecisionBuilder::new("Choose your active pokemon", Player::Player1)
        .indexed_choices(state.player_1.team.iter())
        .build(state, |mut state, choice| {
            state.player_1.active_pokemon_idx = Some(choice);

            DecisionBuilder::new("Choose your active pokemon", Player::Player2)
                .indexed_choices(state.player_2.team.iter())
                .build(state, |mut state, choice| {
                    state.player_2.active_pokemon_idx = Some(choice);
                    Node::end(state)
                })
        })
}
