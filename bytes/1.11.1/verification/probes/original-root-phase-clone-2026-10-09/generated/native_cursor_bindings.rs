unsafe fn inc_start(&mut self, by: usize) {
        // should already be asserted, but debug assert for tests
        debug_assert!(self.len >= by, "internal: inc_start out of bounds");
        self.len -= by;
        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        { self.ptr = self.ptr.add(by); }
        // Selected by == 0: exact same pointer, no new raw access boundary.
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        { proof_assert!(by == 0usize); }
    }

fn remaining(&self) -> usize {
        self.len()
    }

fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

fn advance(&mut self, cnt: usize) {
        assert!(
            cnt <= self.len(),
            "cannot advance past `remaining`: {:?} <= {:?}",
            cnt,
            self.len(),
        );

        unsafe {
            self.inc_start(cnt);
        }
    }

pub const fn len(&self) -> usize {
        self.len
    }
