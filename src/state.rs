use std::{
    fmt::{self, Debug, Formatter},
    ops::Try,
};

pub trait StateBase: Clone + Debug {
    type Player: PlayerBase;
    type PlayerState: PlayerStateBase;

    fn player(&self, player: Self::Player) -> &Self::PlayerState;
    fn player_mut(&mut self, player: Self::Player) -> &mut Self::PlayerState;

    fn fold<I, F>(self, iter: I, f: F) -> Node<Self>
    where
        Self: 'static,
        I: IntoIterator,
        I::Item: Clone + 'static,
        F: Fn(Self, I::Item) -> Node<Self> + Clone + Copy + 'static,
    {
        iter.into_iter()
            .fold(Node::pending(self), move |node, item| {
                node.then(move |state| f(state, item))
            })
    }
}

pub trait PlayerBase: Debug + Clone + Copy {
    fn values() -> &'static [Self];
}

pub trait PlayerStateBase: Debug + Clone {}

pub struct DecisionBuilder<S, T>
where
    S: StateBase,
{
    player: S::Player,
    name: String,
    choices: Vec<(String, T)>,
}

impl<S, T> DecisionBuilder<S, T>
where
    S: StateBase,
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
    S: StateBase,
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
    S: StateBase,
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

impl<S> DecisionBuilder<S, bool>
where
    S: StateBase,
{
    pub fn yn_choices(self) -> Self {
        self.named_choices([("Yes", true), ("No", false)].iter().copied())
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
        S: StateBase,
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

pub trait EventHandler<S>
where
    S: StateBase,
{
    fn on_turn_start(&self, state: S) -> Node<S> {
        Node::pending(state)
    }

    fn on_turn_end(&self, state: S) -> Node<S> {
        Node::pending(state)
    }

    fn on_etb(&self, state: S) -> Node<S> {
        Node::pending(state)
    }
}

#[derive(Debug)]
pub struct Node<S>
where
    S: StateBase,
{
    state: S,
    branches: Branches<S>,
}

impl<S> Node<S>
where
    S: StateBase,
{
    pub fn end(state: S) -> Self {
        Self {
            state,
            branches: Branches::End,
        }
    }

    pub fn pending(state: S) -> Self {
        Self {
            state,
            branches: Branches::Pending,
        }
    }
}

impl<S> Node<S>
where
    S: StateBase + 'static,
{
    pub fn then<F>(self, f: F) -> Self
    where
        F: FnOnce(S) -> Node<S> + Clone + 'static,
    {
        match self.branches {
            Branches::Chance(c) => Self {
                state: self.state,
                branches: Branches::Chance(Chance {
                    name: c.name,
                    possibilities: c
                        .possibilities
                        .into_iter()
                        .map(move |p| {
                            let cont = p.continuation;
                            let f = f.clone();
                            Possibility {
                                name: p.name,
                                weight: p.weight,
                                continuation: Box::new(move |s| cont(s).then(f)),
                            }
                        })
                        .collect(),
                }),
            },
            Branches::Decision(d) => Self {
                state: self.state,
                branches: Branches::Decision(Decision {
                    player: d.player,
                    name: d.name,
                    choices: d
                        .choices
                        .into_iter()
                        .map(move |c| {
                            let cont = c.continuation;
                            let f = f.clone();
                            Choice {
                                name: c.name,
                                continuation: Box::new(move |s| cont(s).then(f)),
                            }
                        })
                        .collect(),
                }),
            },
            Branches::Pending => f(self.state),
            Branches::End => Node::end(self.state),
        }
    }
}

impl<S> Try for Node<S>
where
    S: StateBase + 'static,
{
    type Ok = Self;
    type Error = Self;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match &self.branches {
            Branches::End => Err(self),
            Branches::Chance(_) | Branches::Decision(_) | Branches::Pending => Ok(self),
        }
    }

    fn from_error(v: Self::Ok) -> Self {
        match &v.branches {
            Branches::Chance(_) | Branches::Decision(_) | Branches::Pending => v,
            Branches::End => panic!("Invalid Ok: {:?}", v),
        }
    }

    fn from_ok(v: Self::Error) -> Self {
        match &v.branches {
            Branches::Chance(_) | Branches::Decision(_) | Branches::Pending => {
                panic!("Invalid Error: {:?}", v)
            }
            Branches::End => v,
        }
    }
}

#[derive(Debug)]
pub enum Branches<S>
where
    S: StateBase,
{
    Chance(Chance<S>),
    Decision(Decision<S>),
    Pending,
    End,
}

#[derive(Debug)]
pub struct Chance<S>
where
    S: StateBase,
{
    name: String,
    possibilities: Vec<Possibility<S>>,
}

pub struct Possibility<S>
where
    S: StateBase,
{
    name: String,
    continuation: Box<dyn FnOnce(S) -> Node<S>>,
    weight: f64,
}

impl<S: Debug> Debug for Possibility<S>
where
    S: StateBase,
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
    S: StateBase,
{
    name: String,
    player: S::Player,
    choices: Vec<Choice<S>>,
}

pub struct Choice<S>
where
    S: StateBase,
{
    name: String,
    continuation: Box<dyn FnOnce(S) -> Node<S>>,
}

impl<S: Debug> Debug for Choice<S>
where
    S: StateBase,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Choice").field("name", &self.name).finish()
    }
}
