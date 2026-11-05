# Source type conditions and verified operation

Original iterator.rs224–229 implements Iterator for IterEither<L,R> under L:Iterator and R:Iterator, with Item=Either<L::Item,R::Item>. No same-Item condition exists. This differs from Iterator for Either, which requires matching Item. IntoIterator associated iterator types provide their own Iterator dictionaries, so no fabricated same-Item condition is added to factor_iter's independent item families.

The new theorem verifies only selection, callback Result propagation, and original IterEither construction. It does not verify the returned iterator next implementation or any sequence law. Shared-borrow HRTB origin is retained, and type availability does not establish physical alias/lifetime behavior or arbitrary dictionary law. Reviewer confusion between Either/IterEither was corrected without source/proof changes.
