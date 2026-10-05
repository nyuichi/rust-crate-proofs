#[allow(dead_code)]

pub mod take {
#[cfg(creusot)]
use creusot_std::prelude::*;
pub struct Take<T> {
    inner: T,
    limit: usize,
}
#[cfg_attr(creusot, ensures(result.inner == inner))]
#[cfg_attr(creusot, ensures(result.limit == limit))]
pub fn new<T>(inner: T, limit: usize) -> Take<T> {
    Take { inner, limit }
}
impl<T> Take<T> {
    #[cfg_attr(creusot, ensures(result == self.inner))]
    pub fn into_inner(self) -> T {
        self.inner
    }
    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures(*result == self.inner))]
    pub fn get_ref(&self) -> &T {
        &self.inner
    }
    #[cfg_attr(creusot, ensures(^result == (^self).inner))]
    #[cfg_attr(creusot, ensures((^self).limit == self.limit))]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures(result == self.limit))]
    pub fn limit(&self) -> usize {
        self.limit
    }
    #[cfg_attr(creusot, ensures(self.limit == lim))]
    #[cfg_attr(creusot, ensures((^self).inner == self.inner))]
    pub fn set_limit(&mut self, lim: usize) {
        self.limit = lim
    }
}
}

pub mod limit {
#[cfg(creusot)]
use creusot_std::prelude::*;
pub struct Limit<T> {
    inner: T,
    limit: usize,
}
    #[cfg_attr(creusot, ensures(result.inner == inner))]
    #[cfg_attr(creusot, ensures(result.limit == limit))]
pub(super) fn new<T>(inner: T, limit: usize) -> Limit<T> {
    Limit { inner, limit }
}
impl<T> Limit<T> {
    #[cfg_attr(creusot, ensures(result == self.inner))]
    pub fn into_inner(self) -> T {
        self.inner
    }
    #[cfg_attr(creusot, ensures(*result == self.inner))]
    pub fn get_ref(&self) -> &T {
        &self.inner
    }
    #[cfg_attr(creusot, ensures(^result == (^self).inner))]
    #[cfg_attr(creusot, ensures((^self).limit == self.limit))]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
    #[cfg_attr(creusot, ensures(result == self.limit))]
    pub fn limit(&self) -> usize {
        self.limit
    }
    #[cfg_attr(creusot, ensures(self.limit == lim))]
    #[cfg_attr(creusot, ensures((^self).inner == self.inner))]
    pub fn set_limit(&mut self, lim: usize) {
        self.limit = lim
    }
}

#[cfg_attr(creusot, ensures(result.0 == inner))]
#[cfg_attr(creusot, ensures(result.1@ == new_limit@))]
pub fn metadata_round_trip<T>(inner: T, initial_limit: usize, new_limit: usize) -> (T, usize) {
    let mut value = new(inner, initial_limit);
    let _ = value.get_ref();
    let _ = value.get_mut();
    value.set_limit(new_limit);
    let observed = value.limit();
    let owned = value.into_inner();
    (owned, observed)
}

pub fn new_for_native<T>(inner: T, limit: usize) -> Limit<T> {
    new(inner, limit)
}
}

pub mod chain {
#[cfg(creusot)]
use creusot_std::prelude::*;
pub struct Chain<T, U> {
    a: T,
    b: U,
}
impl<T, U> Chain<T, U> {
    #[cfg_attr(creusot, ensures(result.a == a))]
    #[cfg_attr(creusot, ensures(result.b == b))]
    pub(crate) fn new(a: T, b: U) -> Chain<T, U> {
        Chain { a, b }
    }
    #[cfg_attr(creusot, ensures(result.0 == self.a))]
    #[cfg_attr(creusot, ensures(result.1 == self.b))]
    pub fn into_inner(self) -> (T, U) {
        (self.a, self.b)
    }
}
}
