// Repository-owned standard-library extension for exact slice chunks.
//
// ChunksExact keeps its source, cursor, and chunk size private in libcore.
// The opaque View is therefore the external boundary. The constructor
// contract defines that view as the exact sequence of full chunks plus the
// leftover suffix and the nonzero chunk size; IteratorSpec then defines
// progression by the exact chunks consumed. This element correspondence is
// required by clients that prove functional properties of chunked algorithms,
// rather than only memory and arithmetic safety.

#[logic]
#[requires(chunk_size@ > 0)]
fn chunks_exact_ext_view<'a, T>(
    slice: &'a [T],
    chunk_size: usize,
) -> (Seq<Seq<T>>, Seq<T>, Int) {
    pearlite! {
        let full_len = slice@.len() - slice@.len() % chunk_size@;
        let chunks = Seq::create(full_len / chunk_size@, |index|
            slice@.subsequence(index * chunk_size@, (index + 1) * chunk_size@));
        (chunks, slice@.subsequence(full_len, slice@.len()), chunk_size@)
    }
}

impl<'a, T> View for ChunksExact<'a, T> {
    type ViewTy = (Seq<Seq<T>>, Seq<T>, Int);

    #[trusted]
    #[logic(opaque)]
    fn view(self) -> Self::ViewTy {
        dead
    }
}

impl<'a, T> Invariant for ChunksExact<'a, T> {
    #[logic(open, prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            inv(self@.0) && inv(self@.1) && self@.2 > 0
                && forall<i> 0 <= i && i < self@.0.len() ==> self@.0[i].len() == self@.2
        }
    }
}

impl<'a, T> Resolve for ChunksExact<'a, T> {
    #[logic(open, prophetic)]
    fn resolve(self) -> bool {
        pearlite! { resolve(self@.0) && resolve(self@.1) }
    }

    #[trusted]
    #[logic(prophetic)]
    #[requires(inv(self))]
    #[requires(crate::resolve::structural_resolve(self))]
    #[ensures(self.resolve())]
    fn resolve_coherence(self) {}
}

impl<'a, T> IteratorSpec for ChunksExact<'a, T> {
    #[logic(open, prophetic)]
    fn completed(&mut self) -> bool {
        pearlite! { resolve(self) && (*self)@.0 == Seq::empty() }
    }

    #[logic(open, inline)]
    fn produces(self, visited: Seq<Self::Item>, remaining: Self) -> bool {
        pearlite! {
            0 <= visited.len() && visited.len() <= self@.0.len()
                && visited.len() + remaining@.0.len() == self@.0.len()
                && (remaining@.0 == Seq::empty() ==> visited.len() == self@.0.len())
                && remaining@.0
                    == self@.0.subsequence(visited.len(), self@.0.len())
                && forall<i> 0 <= i && i < visited.len()
                    ==> visited[i]@ == self@.0[i]
                && forall<i> 0 <= i && i < visited.len()
                    ==> visited[i]@.len() == self@.2
                && remaining@.1 == self@.1
                && remaining@.2 == self@.2
        }
    }

    #[logic(law)]
    #[ensures(self.produces(Seq::empty(), self))]
    fn produces_refl(self) {}

    #[logic(law)]
    #[requires(a.produces(ab, b))]
    #[requires(b.produces(bc, c))]
    #[ensures(a.produces(ab.concat(bc), c))]
    fn produces_trans(a: Self, ab: Seq<Self::Item>, b: Self, bc: Seq<Self::Item>, c: Self) {}
}

extern_spec! {
    impl<T> [T] {
        #[check(ghost)]
        #[requires(chunk_size@ > 0)]
        #[ensures(result@ == chunks_exact_ext_view(self, chunk_size))]
        #[ensures(result@.2 == chunk_size@)]
        #[ensures(result@.0.len() == self@.len() / chunk_size@)]
        #[ensures(result@.1.len() == self@.len() % chunk_size@)]
        #[ensures(result@.1.len() < chunk_size@)]
        #[ensures(forall<index> 0 <= index && index < result@.0.len() ==>
            result@.0[index]
                == self@.subsequence(index * chunk_size@, (index + 1) * chunk_size@))]
        #[ensures(self@.len() >= chunk_size@ ==> result@.0 != Seq::empty())]
        fn chunks_exact(&self, chunk_size: usize) -> ChunksExact<'_, T>;
    }

    impl<'a, T> ChunksExact<'a, T> {
        #[check(ghost)]
        #[ensures(result@ == self@.1)]
        fn remainder(&self) -> &'a [T];
    }

    impl<'a, T> Iterator for ChunksExact<'a, T> {
        #[ensures((^self)@.2 == (*self)@.2)]
        #[ensures(match result {
            Some(_) => (^self)@.0.len() + 1 == (*self)@.0.len(),
            None => (^self)@.0.len() == (*self)@.0.len(),
        })]
        #[ensures(match result {
            Some(_) => true,
            None => (*self)@.0 == Seq::empty(),
        })]
        #[ensures(match result {
            None => true,
            Some(chunk) => chunk@.len() == (*self)@.2,
        })]
        fn next(&mut self) -> Option<&'a [T]>;
    }

    impl<T> [T] {
        #[check(ghost)]
        #[requires(mid@ <= self@.len())]
        #[ensures(result.0@ == self@.subsequence(0, mid@))]
        #[ensures(result.1@ == self@.subsequence(mid@, self@.len()))]
        fn split_at(&self, mid: usize) -> (&[T], &[T]);
    }
}
