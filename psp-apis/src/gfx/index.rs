use psp_sys::sys::VertexType;

mod sealed {
    use super::VertexType;
    pub trait IndexItem {}
    impl IndexItem for VertexType {}
}

pub trait IndexItem: sealed::IndexItem {
    fn index_size(&self) -> usize;
}

impl IndexItem for VertexType {
    fn index_size(&self) -> usize {
        if self.contains(VertexType::INDEX_16BIT) {
            2
        } else if self.contains(VertexType::INDEX_8BIT) {
            1
        } else {
            0
        }
    }
}
