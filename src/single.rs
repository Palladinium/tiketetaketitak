use strum_macros::Display;

use crate::{
    pokemon::{Pokemon, Team},
    state::{self, DecisionBuilder, EventHandler, PlayerBase, PlayerStateBase, StateBase},
};

#[derive(Debug, Clone)]
pub struct State {
    player_1: PlayerState,
    player_2: PlayerState,
}

impl StateBase for State {
    type Player = Player;
    type PlayerState = PlayerState;

    fn player(&self, player: Player) -> &PlayerState {
        match player {
            Player::Player1 => &self.player_1,
            Player::Player2 => &self.player_2,
        }
    }

    fn player_mut(&mut self, player: Player) -> &mut PlayerState {
        match player {
            Player::Player1 => &mut self.player_1,
            Player::Player2 => &mut self.player_2,
        }
    }
}

#[derive(Debug, Clone, Copy, Display)]
pub enum Player {
    Player1,
    Player2,
}

impl PlayerBase for Player {
    fn values() -> &'static [Self] {
        &[Player::Player1, Player::Player2]
    }
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub active_pokemon_idx: Option<usize>,
    pub turn_action: Option<Action>,
    pub team: Team,
}

impl PlayerState {
    fn active_pokemon(&self) -> Option<&Pokemon> {
        self.active_pokemon_idx.map(|idx| &self.team[idx])
    }
}

impl PlayerStateBase for PlayerState {}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    UsePokeMove(usize),
    SwitchPokemon(usize),
}

pub type Node = state::Node<State>;

impl State {
    pub fn start(player_1_team: Team, player_2_team: Team) -> Node {
        let state = State {
            player_1: PlayerState {
                team: player_1_team,
                active_pokemon_idx: None,
                turn_action: None,
            },
            player_2: PlayerState {
                team: player_2_team,
                active_pokemon_idx: None,
                turn_action: None,
            },
        };

        state
            .choose_starting_pokemon()
            .then(Self::initial_etb)
            .then(Self::main_turn)
    }

    fn choose_starting_pokemon(self) -> Node {
        self.fold(Player::values(), move |state, &player| {
            DecisionBuilder::new("Choose your active pokemon", player)
                .indexed_choices(state.player(player).team.iter())
                .build(state, move |mut state, choice| {
                    state.player_mut(player).active_pokemon_idx = Some(choice);
                    Node::pending(state)
                })
        })
    }

    fn initial_etb(self) -> Node {
        self.fold(Player::values(), |state, &player| state.pokemon_etb(player))
    }

    fn main_turn(self) -> Node {
        self.choose_actions()
            .then(Self::execute_actions)
            .then(Self::main_turn)
    }

    fn choose_actions(self) -> Node {
        self.fold(Player::values(), |state, &player| {
            let player_state = state.player(player);
            let active_pokemon_idx = player_state.active_pokemon_idx.unwrap_or_else(|| {
                panic!("No active pokemon for {} when choosing actions", player)
            });

            let move_choices = player_state.team[active_pokemon_idx]
                .moves
                .iter()
                .enumerate()
                .map(|(i, m)| (m.to_string(), Action::UsePokeMove(i)));

            let switch_choices = player_state
                .team
                .iter()
                .enumerate()
                .filter(|(i, p)| *i != active_pokemon_idx && p.current_hp > 0)
                .map(|(i, p)| (p.to_string(), Action::SwitchPokemon(i)));

            DecisionBuilder::new("Choose your action", player)
                .named_choices(move_choices.chain(switch_choices))
                .build(state, move |mut state, choice| {
                    state.player_mut(player).turn_action = Some(choice);
                    Node::pending(state)
                })
        })
    }

    fn execute_actions(self) -> Node {
        Node::pending(self)
    }

    fn pokemon_etb(self, player: Player) -> Node {
        // ETB abilities
        self.player(player)
            .active_pokemon()
            .unwrap()
            .ability
            .clone()
            .on_etb(self)
    }
}
