use std::fmt::{self, Debug, Formatter};

use arrayvec::ArrayVec;

use crate::pokemon::Pokemon;

pub trait State: Clone {
    type Player: Debug + Clone;

    fn active_pokemon(&self) -> ActivePokemon<'_>;

    fn chance<N, I, T, S, F>(self, name: N, possibilities: I, f: F) -> Node<Self>
    where
        N: Into<String>,
        I: IntoIterator<Item = (f64, T)> + 'static,
        T: ToString + Clone + 'static,
        F: Fn(Self, T) -> Node<Self> + Clone + 'static,
    {
        Node {
            state: self,
            branches: Branches::Chance(Chance {
                name: name.into(),
                possibilities: possibilities
                    .into_iter()
                    .map(|(weight, p)| {
                        let f = f.clone();
                        Possibility {
                            name: p.to_string(),
                            weight,
                            continuation: Box::new(move |s| f(s, p.clone())),
                        }
                    })
                    .collect(),
            }),
        }
    }
}

pub type ActivePokemon<'a> = ArrayVec<[&'a Pokemon; 4]>;

pub struct DecisionBuilder<S, T>
where
    S: State,
{
    player: S::Player,
    name: String,
    choices: Vec<(String, T)>,
}

impl<S, T> DecisionBuilder<S, T>
where
    S: State,
    T: 'static,
{
    pub fn new<N>(name: N, player: S::Player) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            player,
            choices: Vec::new(),
        }
    }

    pub fn named_choice<N>(mut self, name: N, choice: T) -> Self
    where
        N: Into<String>,
    {
        self.choices.push((name.into(), choice));
        self
    }

    pub fn named_choices<N, I>(mut self, choices: I) -> Self
    where
        N: Into<String>,
        I: IntoIterator<Item = (N, T)>,
    {
        self.choices
            .extend(choices.into_iter().map(|(n, t)| (n.into(), t)));
        self
    }

    pub fn build<F>(self, state: S, f: F) -> Node<S>
    where
        F: FnOnce(S, T) -> Node<S> + Clone + 'static,
    {
        Node {
            state,
            branches: Branches::Decision(Decision {
                name: self.name,
                player: self.player,
                choices: self
                    .choices
                    .into_iter()
                    .map(|(name, c)| {
                        let f = f.clone();
                        Choice {
                            name,
                            continuation: Box::new(move |s| f(s, c)),
                        }
                    })
                    .collect(),
            }),
        }
    }
}

impl<S, T> DecisionBuilder<S, T>
where
    S: State,
    T: ToString + 'static,
{
    pub fn choice(self, choice: T) -> Self {
        self.named_choice(choice.to_string(), choice)
    }

    pub fn choices<I>(self, choices: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.named_choices(choices.into_iter().map(|t| (t.to_string(), t)))
    }
}

impl<S> DecisionBuilder<S, usize>
where
    S: State,
{
    pub fn indexed_choices<I>(self, choices: I) -> Self
    where
        I: IntoIterator,
        I::Item: ToString,
    {
        self.named_choices(
            choices
                .into_iter()
                .map(|c| c.to_string())
                .enumerate()
                .map(|(i, n)| (n, i)),
        )
    }
}

pub struct ChanceBuilder<T> {
    name: String,
    possibilities: Vec<(String, f64, T)>,
}

impl<T> ChanceBuilder<T>
where
    T: 'static,
{
    pub fn named_possibility<N>(mut self, name: N, weight: f64, possibility: T) -> Self
    where
        N: Into<String>,
    {
        self.possibilities.push((name.into(), weight, possibility));
        self
    }

    pub fn named_possibilities<N, I>(mut self, choices: I) -> Self
    where
        N: Into<String>,
        I: IntoIterator<Item = (N, f64, T)>,
    {
        self.possibilities
            .extend(choices.into_iter().map(|(n, w, t)| (n.into(), w, t)));
        self
    }

    pub fn build<S, F>(self, state: S, f: F) -> Node<S>
    where
        S: State,
        F: FnOnce(S, T) -> Node<S> + Clone + 'static,
    {
        Node {
            state: state,
            branches: Branches::Chance(Chance {
                name: self.name,

                possibilities: self
                    .possibilities
                    .into_iter()
                    .map(|(name, weight, p)| {
                        let f = f.clone();
                        Possibility {
                            name,
                            weight,
                            continuation: Box::new(move |s| f(s, p)),
                        }
                    })
                    .collect(),
            }),
        }
    }
}

impl<T> ChanceBuilder<T>
where
    T: ToString + 'static,
{
    pub fn possibility(self, weight: f64, possibility: T) -> Self {
        self.named_possibility(possibility.to_string(), weight, possibility)
    }

    pub fn possibilities<I>(self, possibilities: I) -> Self
    where
        I: IntoIterator<Item = (f64, T)>,
    {
        self.named_possibilities(
            possibilities
                .into_iter()
                .map(|(w, t)| (t.to_string(), w, t)),
        )
    }
}

pub trait Callbacks<S>
where
    S: State,
{
    fn on_turn_start(&self, state: &S) -> Option<Node<S>> {
        None
    }

    fn on_turn_end(&self, state: &S) -> Option<Node<S>> {
        None
    }
}

#[derive(Debug)]
pub struct Node<S>
where
    S: State,
{
    state: S,
    branches: Branches<S>,
}

impl<S> Node<S>
where
    S: State,
{
    pub fn end(state: S) -> Self {
        Self {
            state,
            branches: Branches::End,
        }
    }
}

#[derive(Debug)]
pub enum Branches<S>
where
    S: State,
{
    Chance(Chance<S>),
    Decision(Decision<S>),
    End,
}

#[derive(Debug)]
pub struct Chance<S>
where
    S: State,
{
    name: String,
    possibilities: Vec<Possibility<S>>,
}

pub struct Possibility<S>
where
    S: State,
{
    name: String,
    continuation: Box<dyn FnOnce(S) -> Node<S>>,
    weight: f64,
}

impl<S: Debug> Debug for Possibility<S>
where
    S: State,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Possibility")
            .field("name", &self.name)
            .field("weight", &self.weight)
            .finish()
    }
}

#[derive(Debug)]
pub struct Decision<S>
where
    S: State,
{
    name: String,
    player: S::Player,
    choices: Vec<Choice<S>>,
}

pub struct Choice<S>
where
    S: State,
{
    name: String,
    continuation: Box<dyn FnOnce(S) -> Node<S>>,
}

impl<S: Debug> Debug for Choice<S>
where
    S: State,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Choice").field("name", &self.name).finish()
    }
}
