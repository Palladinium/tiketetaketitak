use crate::state::{Callbacks, Node, State};

pub enum Effect {}

impl<S: State> Callbacks<S> for Effect {}
