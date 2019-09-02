use crate::state::{EventHandler, StateBase};

#[derive(Debug, Clone, Copy)]
pub enum Ability {}

impl<S: StateBase> EventHandler<S> for Ability {}
