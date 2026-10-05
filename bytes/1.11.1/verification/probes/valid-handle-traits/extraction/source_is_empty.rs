    // BEGIN EXACT IS_EMPTY
    #[cfg_attr(creusot, ensures(result == (self.len == 0usize)))]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }