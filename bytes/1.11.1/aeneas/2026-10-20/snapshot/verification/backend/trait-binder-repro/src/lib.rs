pub trait Chain: Sized {
    fn chain<U>(self, next: U) -> (Self, U) {
        (self, next)
    }
}

pub struct Item;
impl Chain for Item {}

pub fn call<U>(item: Item, next: U) -> (Item, U) {
    item.chain(next)
}
