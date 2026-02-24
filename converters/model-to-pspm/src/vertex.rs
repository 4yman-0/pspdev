bitflags::bitflags! {
    /// The vertex type decides how the vertices align and what kind of
    /// information they contain.
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub struct VertexType: i32 {
        /// 8-bit texture coordinates
        const TEXTURE_8BIT = 1;
        /// 16-bit texture coordinates
        const TEXTURE_16BIT = 2;
        /// 32-bit texture coordinates (float)
        const TEXTURE_32BITF = 3;

        /// 16-bit color (R5G6B5A0)
        const COLOR_5650 = 4 << 2;
        /// 16-bit color (R5G5B5A1)
        const COLOR_5551 = 5 << 2;
        /// 16-bit color (R4G4B4A4)
        const COLOR_4444 = 6 << 2;
        /// 32-bit color (R8G8B8A8)
        const COLOR_8888 = 7 << 2;

        /// 8-bit normals
        const NORMAL_8BIT = 1 << 5;
        /// 16-bit normals
        const NORMAL_16BIT = 2 << 5;
        /// 32-bit normals (float)
        const NORMAL_32BITF = 3 << 5;

        /// 8-bit vertex position
        const VERTEX_8BIT = 1 << 7;
        /// 16-bit vertex position
        const VERTEX_16BIT = 2 << 7;
        /// 32-bit vertex position (float)
        const VERTEX_32BITF = 3 << 7;

        /// 8-bit weights
        const WEIGHT_8BIT = 1 << 9;
        /// 16-bit weights
        const WEIGHT_16BIT = 2 << 9;
        /// 32-bit weights (float)
        const WEIGHT_32BITF = 3 << 9;

        /// 8-bit vertex index
        const INDEX_8BIT = 1 << 11;
        /// 16-bit vertex index
        const INDEX_16BIT = 2 << 11;

        // FIXME: Need to document this.
        // Number of weights (1-8)
        const WEIGHTS1 = Self::num_weights(1);
        const WEIGHTS2 = Self::num_weights(2);
        const WEIGHTS3 = Self::num_weights(3);
        const WEIGHTS4 = Self::num_weights(4);
        const WEIGHTS5 = Self::num_weights(5);
        const WEIGHTS6 = Self::num_weights(6);
        const WEIGHTS7 = Self::num_weights(7);
        const WEIGHTS8 = Self::num_weights(8);

        // Number of vertices (1-8)
        const VERTICES1 = Self::num_vertices(1);
        const VERTICES2 = Self::num_vertices(2);
        const VERTICES3 = Self::num_vertices(3);
        const VERTICES4 = Self::num_vertices(4);
        const VERTICES5 = Self::num_vertices(5);
        const VERTICES6 = Self::num_vertices(6);
        const VERTICES7 = Self::num_vertices(7);
        const VERTICES8 = Self::num_vertices(8);

        /// Coordinate is passed directly to the rasterizer
        const TRANSFORM_2D = 1 << 23;
        /// Coordinate is transformed before being passed to rasterizer
        const TRANSFORM_3D = 0;
    }
}

impl VertexType {
    pub const fn num_weights(n: u32) -> i32 {
        (((n - 1) & 7) << 14) as i32
    }

    pub const fn num_vertices(n: u32) -> i32 {
        (((n - 1) & 7) << 18) as i32
    }
}

pub(crate) const fn vt_element_sizes(vt: VertexType) -> [usize; 5] {
    use VertexType as VertType;
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

pub(crate) const fn vt_sizes(element_sizes: &[usize; 5]) -> [usize; 5] {
    let mut arr = *element_sizes;
    arr[0] *= 2;
    arr[2] *= 3;
    arr[3] *= 3;
    arr
}

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

/*pub(crate) const fn vt_size(
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
    size
}*/
