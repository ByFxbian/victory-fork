use super::*;

impl Lower<'_> {
    fn quad_across(&mut self, x: V, xor: u8) -> V {
        self.s.flags |= flags::WAVE_OPS;
        let direction = self.s.m.i8(xor - 1);
        self.s.op(
            self.cur,
            "quadOp.i32",
            Attr::NoUnwind,
            self.s.t.i32,
            123,
            &[x, direction],
        )
    }

    pub(super) fn pixel_quad_mask(&mut self, input: Bool) -> Result<V, Why> {
        let c = self.cur;
        let on = self.boolean(input, c)?;
        if matches!(input, Bool::TRUE | Bool::FALSE) {
            return Ok(on);
        }
        let bits = self.s.m.cast(c, Cast::ZExt, on, self.s.t.i32);
        let across = self.quad_across(bits, 1);
        let pair = self.s.m.bin(c, Bin::Or, bits, across);
        let other = self.quad_across(pair, 2);
        let all = self.s.m.bin(c, Bin::Or, pair, other);
        let zero = self.s.m.i32(0);
        Ok(self.s.m.cmp(c, pred::NE, all, zero))
    }

    /// `x` of lane `src` (in this lane's quad) through quad reads, whose
    /// lane must be a constant.
    pub(super) fn quad_pick(&mut self, x: V, src: V, pattern: Option<u8>) -> V {
        self.s.flags |= flags::WAVE_OPS;
        let c = self.cur;
        // Native patterns from DexNom/VIctory 921d8d3b.
        match pattern {
            Some(0xe4) => return x,
            Some(p @ (0x00 | 0x55 | 0xaa | 0xff)) => {
                let lane = self.s.m.i32(u32::from(p & 3));
                return self.s.op(
                    c,
                    "quadReadLaneAt.i32",
                    Attr::NoUnwind,
                    self.s.t.i32,
                    122,
                    &[x, lane],
                );
            }
            Some(p @ (0xb1 | 0x4e | 0x1b)) => return self.quad_across(x, p & 3),
            _ => {}
        }
        let three = self.s.m.i32(3);
        let q = self.s.m.bin(c, Bin::And, src, three);
        let mut r = x;
        for n in 0..4 {
            let kn = self.s.m.i32(n);
            let read = self.s.op(
                c,
                "quadReadLaneAt.i32",
                Attr::NoUnwind,
                self.s.t.i32,
                122,
                &[x, kn],
            );
            let is = self.s.m.cmp(c, pred::EQ, q, kn);
            r = self.s.m.select(c, is, read, r);
        }
        r
    }
}
