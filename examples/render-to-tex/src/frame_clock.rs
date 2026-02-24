#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct FrameClock(usize);

#[allow(dead_code)]
impl FrameClock {
    #[must_use]
    pub(crate) const fn update(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    #[must_use]
    pub(crate) const fn continous_clock(self, half_period: usize) -> bool {
        self.0 % half_period > (half_period / 2 - 1)
    }
    #[must_use]
    pub(crate) const fn edge_clock(self, period: usize) -> bool {
        self.0.is_multiple_of(period)
    }
    #[must_use]
    pub(crate) const fn offset(self, offset: usize) -> Self {
        Self(self.0.wrapping_add(offset))
    }
}
