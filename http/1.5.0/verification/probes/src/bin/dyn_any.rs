#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;
use std::any::Any;

trait AnyClone: Any {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync>;
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Clone + Send + Sync> AnyClone for T {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn main() {}
