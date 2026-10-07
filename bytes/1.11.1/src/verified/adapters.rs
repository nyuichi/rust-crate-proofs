//! Checked limit and concatenation adapters over immutable cursor borrows.
//! Scoped clients retain the original read lease through explicit retirement.
use super::{cursor::Cursor, exclusive::ExclusiveBytes};
use alloc::vec::Vec;
use creusot_std::prelude::*;

pub struct LimitedCursor<'a> {
    inner: Cursor<'a>,
    limit: usize,
}
impl<'a> View for LimitedCursor<'a> {
    type ViewTy = Seq<u8>;
    #[logic]
    fn view(self) -> Seq<u8> {
        pearlite! { self.inner@[0..self.limit@.min(self.inner@.len())] }
    }
}
impl<'a> LimitedCursor<'a> {
    #[logic]
    pub fn inner_view(self) -> Seq<u8> {
        pearlite! { self.inner@ }
    }
    #[logic]
    pub fn limit_view(self) -> Int {
        pearlite! { self.limit@ }
    }
    #[ensures(result.inner_view() == inner@)]
    #[ensures(result.limit_view() == limit@)]
    #[ensures(result@ == inner@[0..limit@.min(inner@.len())])]
    pub fn new(inner: Cursor<'a>, limit: usize) -> Self {
        Self { inner, limit }
    }
    #[ensures(result@ == self.limit_view())]
    pub fn limit(&self) -> usize {
        self.limit
    }
    #[ensures((^self).limit_view() == limit@)]
    #[ensures((^self).inner_view() == self.inner_view())]
    #[ensures((^self)@ == self.inner_view()[0..limit@.min(self.inner_view().len())])]
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }
    #[ensures(result@ == self.inner_view())]
    pub fn into_inner(self) -> Cursor<'a> {
        self.inner
    }
    #[ensures(result@ == self@.len())]
    pub fn remaining(&self) -> usize {
        let n = self.inner.remaining();
        if n < self.limit {
            n
        } else {
            self.limit
        }
    }
    #[ensures(result@ == self@)]
    pub fn chunk(&self) -> &'a [u8] {
        &self.inner.chunk()[..self.remaining()]
    }
    #[ensures(result == (count@ <= self@.len()))]
    #[ensures(if result {
        (^self)@ == self@[count@..] &&
        (^self).inner_view() == self.inner_view()[count@..] &&
        (^self).limit_view() == self.limit_view()-count@
    } else { (^self)@ == self@ && (^self).inner_view() == self.inner_view() && (^self).limit_view() == self.limit_view() })]
    pub fn advance(&mut self, count: usize) -> bool {
        if count > self.remaining() {
            false
        } else {
            let ok = self.inner.advance(count);
            proof_assert!(ok);
            self.limit -= count;
            true
        }
    }
    #[ensures(result == (dst@.len() <= self@.len()))]
    #[ensures(if result { (^dst)@ == self@[0..dst@.len()] && (^self)@ == self@[dst@.len()..] }
        else { (^dst)@ == dst@ && (^self)@ == self@ })]
    #[ensures(if result {
        (^self).inner_view() == self.inner_view()[dst@.len()..] &&
        (^self).limit_view() == self.limit_view()-dst@.len()
    } else { (^self).inner_view() == self.inner_view() && (^self).limit_view() == self.limit_view() })]
    pub fn copy_to_slice(&mut self, dst: &mut [u8]) -> bool {
        if dst.len() > self.remaining() {
            false
        } else {
            let n = dst.len();
            let ok = self.inner.copy_to_slice(dst);
            proof_assert!(ok);
            self.limit -= n;
            true
        }
    }
    #[ensures(match result {
        Some(v)=>self@.len()>=2 && v@ == self@[0]@*256+self@[1]@ && (^self)@ == self@[2..],
        None=>self@.len()<2 && (^self)@ == self@
    })]
    #[ensures(if result != None {
        (^self).inner_view() == self.inner_view()[2..] &&
        (^self).limit_view() == self.limit_view()-2
    } else { (^self).inner_view() == self.inner_view() && (^self).limit_view() == self.limit_view() })]
    pub fn try_get_u16_be(&mut self) -> Option<u16> {
        if self.remaining() < 2 {
            None
        } else {
            let result = self.inner.try_get_u16_be();
            self.limit -= 2;
            result
        }
    }
}

/// Rejoins the two copied pieces in a context containing only sequence facts.
#[cfg_attr(creusot, check(ghost))]
#[requires(0 <= *middle && *middle <= filled.len())]
#[requires(filled.len() <= original.len())]
#[requires(filled[0..*middle] == original[0..*middle])]
#[requires(filled[*middle..] == original[*middle..filled.len()])]
#[ensures(*filled == original[0..filled.len()])]
fn rejoin_prefix(original: Snapshot<Seq<u8>>, filled: Snapshot<Seq<u8>>, middle: Snapshot<Int>) {
    proof_assert!(forall<i: Int> 0 <= i && i < filled.len() ==>
    if i < *middle {
        filled[0..*middle][i] == original[0..*middle][i]
            && filled[i] == original[i]
    } else {
        filled[*middle..][i - *middle] == original[*middle..filled.len()][i - *middle]
            && filled[i] == original[i]
    });
    proof_assert!(filled.ext_eq(original[0..filled.len()]));
}

pub struct ChainedCursor<'a> {
    left: Cursor<'a>,
    right: Cursor<'a>,
}
impl<'a> View for ChainedCursor<'a> {
    type ViewTy = Seq<u8>;
    #[logic]
    fn view(self) -> Seq<u8> {
        pearlite! { self.left@.concat(self.right@) }
    }
}
impl<'a> ChainedCursor<'a> {
    #[ensures(result@ == left@.concat(right@))]
    pub fn new(left: Cursor<'a>, right: Cursor<'a>) -> Self {
        Self { left, right }
    }
    #[ensures(result.0@.concat(result.1@) == self@)]
    pub fn into_inner(self) -> (Cursor<'a>, Cursor<'a>) {
        (self.left, self.right)
    }
    /// Saturates if the mathematical total exceeds usize::MAX.
    #[ensures(result@ == self@.len().min(usize::MAX@))]
    pub fn remaining(&self) -> usize {
        let left = self.left.remaining();
        let right = self.right.remaining();
        if usize::MAX - left < right {
            usize::MAX
        } else {
            left + right
        }
    }
    #[ensures(result@ == self@[0..result@.len()])]
    #[ensures((result@.len()==0) == (self@.len()==0))]
    pub fn chunk(&self) -> &'a [u8] {
        if self.left.remaining() == 0 {
            self.right.chunk()
        } else {
            self.left.chunk()
        }
    }
    #[ensures(result == (count@ <= self@.len()))]
    #[ensures(if result {(^self)@ == self@[count@..]} else {(^self)@ == self@})]
    pub fn advance(&mut self, count: usize) -> bool {
        let left = self.left.remaining();
        if count <= left {
            self.left.advance(count)
        } else {
            let rest = count - left;
            if rest > self.right.remaining() {
                false
            } else {
                let ok = self.left.advance(left);
                proof_assert!(ok);
                let ok = self.right.advance(rest);
                proof_assert!(ok);
                true
            }
        }
    }
    #[ensures(result == (dst@.len() <= self@.len()))]
    #[ensures(if result { (^dst)@ == self@[0..dst@.len()] && (^self)@ == self@[dst@.len()..] }
        else {(^dst)@ == dst@ && (^self)@ == self@})]
    pub fn copy_to_slice(&mut self, dst: &mut [u8]) -> bool {
        let original = snapshot! { self@ };
        let left = self.left.remaining();
        if dst.len() <= left {
            self.left.copy_to_slice(dst)
        } else {
            let rest = dst.len() - left;
            if rest > self.right.remaining() {
                false
            } else {
                let (a, b) = dst.split_at_mut(left);
                let ok = self.left.copy_to_slice(a);
                proof_assert!(ok);
                let ok = self.right.copy_to_slice(b);
                proof_assert!(ok);
                // Discharge reassembly outside the mutable-borrow context.
                ghost! { rejoin_prefix(original, snapshot! { dst@ }, snapshot! { left@ }); };
                true
            }
        }
    }
    #[ensures(match result {
        Some(v)=>self@.len()>=2 && v@ == self@[0]@*256+self@[1]@ && (^self)@ == self@[2..],
        None=>self@.len()<2 && (^self)@ == self@
    })]
    pub fn try_get_u16_be(&mut self) -> Option<u16> {
        let mut encoded = [0u8; 2];
        if self.copy_to_slice(&mut encoded) {
            Some(crate::byte_codec_ops::decode_be_u16(encoded))
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
#[ensures(match result.0 {
    Some(v)=>input@.len().min(limit@)>=2 && v@ == input@[0]@*256+input@[1]@ && result.1@ == input@.len().min(limit@)-2,
    None=>input@.len().min(limit@)<2 && result.1@ == input@.len().min(limit@)
})]
pub fn scoped_limited_read(input: Vec<u8>, limit: usize) -> (Option<u16>, usize) {
    super::with_shared_read(
        input,
        move |bytes: &[u8]| {
            let mut cursor = LimitedCursor::new(Cursor::new(bytes), limit);
            let result = cursor.try_get_u16_be();
            (result, cursor.remaining())
        },
        |_: &[u8]| (),
    )
    .0
}

#[cfg(feature = "std")]
#[ensures((result == None) == (split@ > input@.len()))]
#[ensures(result != None ==> match result.unwrap_logic().0 {
    Some(v)=>input@.len()>=2 && v@ == input@[0]@*256+input@[1]@ && result.unwrap_logic().1@ == input@.len()-2,
    None=>input@.len()<2 && result.unwrap_logic().1@ == input@.len()
})]
pub fn scoped_chained_read(input: Vec<u8>, split: usize) -> Option<(Option<u16>, usize)> {
    if split > input.len() {
        ExclusiveBytes::from_vec(input).close();
        return None;
    }
    Some(
        super::with_shared_read(
            input,
            move |bytes: &[u8]| {
                let mut cursor =
                    ChainedCursor::new(Cursor::new(&bytes[..split]), Cursor::new(&bytes[split..]));
                let result = cursor.try_get_u16_be();
                (result, cursor.remaining())
            },
            |_: &[u8]| (),
        )
        .0,
    )
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::*;
    #[test]
    fn limited_cursor_bounds_and_underlying_suffix() {
        let bytes = [0x12, 0x34, 0x56, 0x78];
        for limit in 0..7 {
            let mut cursor = LimitedCursor::new(Cursor::new(&bytes), limit);
            let exposed = limit.min(bytes.len());
            assert_eq!(cursor.chunk(), &bytes[..exposed]);
            let value = cursor.try_get_u16_be();
            assert_eq!(value, if exposed >= 2 { Some(0x1234) } else { None });
            let consumed = if value.is_some() { 2 } else { 0 };
            assert_eq!(cursor.remaining(), exposed - consumed);
            assert_eq!(cursor.limit(), limit - consumed);
            assert_eq!(cursor.into_inner().chunk(), &bytes[consumed..]);
        }
    }
    #[test]
    fn chained_cursor_crosses_boundary_and_preserves_failed_destination() {
        let bytes = [0x12, 0x34, 0x56, 0x78];
        for split in 0..=bytes.len() {
            for count in 0..7 {
                let mut cursor =
                    ChainedCursor::new(Cursor::new(&bytes[..split]), Cursor::new(&bytes[split..]));
                let mut target = [0xa5; 6];
                let success = cursor.copy_to_slice(&mut target[..count]);
                assert_eq!(success, count <= bytes.len());
                if success {
                    assert_eq!(&target[..count], &bytes[..count]);
                    assert_eq!(cursor.remaining(), bytes.len() - count);
                } else {
                    assert_eq!(target, [0xa5; 6]);
                    assert_eq!(cursor.remaining(), bytes.len());
                }
                let mut numeric =
                    ChainedCursor::new(Cursor::new(&bytes[..split]), Cursor::new(&bytes[split..]));
                assert_eq!(numeric.try_get_u16_be(), Some(0x1234));
                assert_eq!(numeric.remaining(), 2);
            }
        }
    }
    #[cfg(feature = "std")]
    #[test]
    fn adapters_read_actual_shared_storage_and_close() {
        for len in 0..6 {
            for split in 0..7 {
                for limit in 0..7 {
                    let input: Vec<u8> = (0..len).map(|i| 17 + i as u8).collect();
                    let expected = if len >= 2 { Some(17 * 256 + 18) } else { None };
                    let selected = len.min(limit);
                    let limited = if selected >= 2 {
                        Some(17 * 256 + 18)
                    } else {
                        None
                    };
                    assert_eq!(
                        scoped_limited_read(input.clone(), limit),
                        (limited, selected - if limited.is_some() { 2 } else { 0 })
                    );
                    assert_eq!(
                        scoped_chained_read(input, split),
                        if split <= len {
                            Some((expected, len - if expected.is_some() { 2 } else { 0 }))
                        } else {
                            None
                        }
                    );
                }
            }
        }
    }
}
