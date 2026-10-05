#[derive(PartialEq)]
#[cfg_attr(creusot, derive(creusot_std::prelude::DeepModel))]
pub struct Generic<T>(pub T);
