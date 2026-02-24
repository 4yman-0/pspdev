use psp_sys::sys::VertexType as VertType;

pub(crate) const fn vt_element_sizes(vt: VertType) -> [usize; 5] {
    [
        if vt.contains(VertType::TEXTURE_32BITF) {
            4
        } else if vt.contains(VertType::TEXTURE_8BIT) {
            1
        } else if vt.contains(VertType::TEXTURE_16BIT) {
            2
        } else {
            0
        },
        if vt.contains(VertType::COLOR_8888) {
            4
        } else if vt.contains(VertType::COLOR_5650)
            || vt.contains(VertType::COLOR_5551)
            || vt.contains(VertType::COLOR_4444)
        {
            2
        } else {
            0
        },
        if vt.contains(VertType::NORMAL_32BITF) {
            4
        } else if vt.contains(VertType::NORMAL_8BIT) {
            1
        } else if vt.contains(VertType::NORMAL_16BIT) {
            2
        } else {
            0
        },
        if vt.contains(VertType::VERTEX_32BITF) {
            4
        } else if vt.contains(VertType::VERTEX_8BIT) {
            1
        } else if vt.contains(VertType::VERTEX_16BIT) {
            2
        } else {
            0
        },
        if vt.contains(VertType::WEIGHT_32BITF) {
            4
        } else if vt.contains(VertType::WEIGHT_8BIT) {
            1
        } else if vt.contains(VertType::WEIGHT_16BIT) {
            2
        } else {
            0
        },
    ]
}

pub(crate) const fn vt_sizes(
    element_sizes: &[usize; 5],
    vt: VertType,
) -> [usize; 5] {
    let mut arr = *element_sizes;
    arr[0] *= 2;
    arr[2] *= 3;
    arr[3] *= 3;
    arr[4] *= if vt.contains(VertType::WEIGHTS3) {
        3
    } else if vt.contains(VertType::WEIGHTS2) {
        2
    } else {
        1
    };
    arr
}

#[allow(dead_code)]
pub(crate) const fn vt_alignments(
    elem_sizes: &[usize; 5],
    sizes: &[usize; 5],
) -> (usize, [usize; 5]) {
    let mut arr = [0usize; 5];
    let mut size = 0usize;
    let mut i = 0;
    while i < 5 {
        let (a, s) = (elem_sizes[i], sizes[i]);
        if a > 1 && !size.is_multiple_of(a) {
            let align = a - (size % a);
            arr[i] = align;
            size += align;
        }
        size += s;
        i += 1;
    }
    (size, arr)
}

pub(crate) const fn vt_size(
    elem_sizes: &[usize; 5],
    sizes: &[usize; 5],
) -> usize {
    let mut size = 0usize;
    let mut i = 0;
    while i < 5 {
        let (a, s) = (elem_sizes[i], sizes[i]);
        if a > 1 && !size.is_multiple_of(a) {
            let align = a - (size % a);
            size += align;
        }
        size += s;
        i += 1;
    }
    // hacks...
    match size {
        10 => 12,
        _ => size,
    }
}

pub const fn const_vt_size(vertex_type: VertType) -> VertexSize {
    let elem_sizes = vt_element_sizes(vertex_type);
    let sizes = vt_sizes(&elem_sizes, vertex_type);
    VertexSize(vt_size(&elem_sizes, &sizes))
}

mod sealed {
    use super::VertType;
    pub trait Vertex {}
    impl Vertex for VertType {}
}

pub trait Vertex: sealed::Vertex {
    fn vertex_size(&self) -> VertexSize;
}

impl Vertex for VertType {
    fn vertex_size(&self) -> VertexSize {
        let elem_sizes = vt_element_sizes(*self);
        let sizes = vt_sizes(&elem_sizes, *self);
        VertexSize(vt_size(&elem_sizes, &sizes))
    }
}

#[derive(Clone, Copy)]
pub struct VertexSize(usize);

impl VertexSize {
    pub fn get(&self) -> usize {
        self.0
    }
}
