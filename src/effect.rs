use crate::state::{EventHandler, StateBase};

#[derive(Debug, Clone)]
pub enum Effect {}

impl<S: StateBase> EventHandler<S> for Effect {}
