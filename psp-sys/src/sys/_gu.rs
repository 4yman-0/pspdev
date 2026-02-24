#![allow(static_mut_refs)]

use core::ffi::c_void;

/// Primitive types
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum GuPrimitive {
    /// Single pixel points (1 vertex per primitive)
    Points = 0,
    /// Single pixel lines (2 vertices per primitive)
    Lines = 1,
    /// Single pixel line-strip (2 vertices for the first primitive, 1 for every following)
    LineStrip = 2,
    /// Filled triangles (3 vertices per primitive)
    Triangles = 3,
    /// Filled triangles-strip (3 vertices for the first primitive, 1 for every following)
    TriangleStrip = 4,
    /// Filled triangle-fan (3 vertices for the first primitive, 1 for every following)
    TriangleFan = 5,
    /// Filled blocks (2 vertices per primitive)
    Sprites = 6,
}

/// Patch primitive types
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum PatchPrimitive {
    /// Single pixel points (1 vertex per primitive)
    Points = 0,
    /// Single pixel line-strip (2 vertices for the first primitive, 1 for every following)
    LineStrip = 2,
    /// Filled triangles-strip (3 vertices for the first primitive, 1 for every following)
    TriangleStrip = 4,
}

/// States
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum GuState {
    AlphaTest = 0,
    DepthTest = 1,
    ScissorTest = 2,
    StencilTest = 3,
    Blend = 4,
    CullFace = 5,
    Dither = 6,
    Fog = 7,
    ClipPlanes = 8,
    Texture2D = 9,
    Lighting = 10,
    Light0 = 11,
    Light1 = 12,
    Light2 = 13,
    Light3 = 14,
    LineSmooth = 15,
    PatchCullFace = 16,
    ColorTest = 17,
    ColorLogicOp = 18,
    FaceNormalReverse = 19,
    PatchFace = 20,
    Fragment2X = 21,
}

/// Matrix modes
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum MatrixMode {
    Projection = 0,
    View = 1,
    Model = 2,
    Texture = 3,
}

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

/// Texture pixel formats
// TODO: Better documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TexturePixelFormat {
    /// Hicolor, 16-bit.
    Psm5650 = 0,
    /// Hicolor, 16-bit
    Psm5551 = 1,
    /// Hicolor, 16-bit
    Psm4444 = 2,
    /// Truecolor, 32-bit
    Psm8888 = 3,
    /// Indexed, 4-bit (2 pixels per byte)
    PsmT4 = 4,
    /// Indexed, 8-bit
    PsmT8 = 5,
    /// Indexed, 16-bit
    PsmT16 = 6,
    /// Indexed, 32-bit
    PsmT32 = 7,
    PsmDxt1 = 8,
    PsmDxt3 = 9,
    PsmDxt5 = 10,
}

/// Spline Mode
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum SplineMode {
    FillFill = 0,
    OpenFill = 1,
    FillOpen = 2,
    OpenOpen = 3,
}

/// Shading Model
#[repr(u32)]
// TODO: Should this be `ShadeMode` (no L)?
#[derive(Clone, Copy)]
pub enum ShadingModel {
    Flat = 0,
    Smooth = 1,
}

/// Logical operation
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum LogicalOperation {
    Clear = 0,
    And = 1,
    AndReverse = 2,
    Copy = 3,
    AndInverted = 4,
    Noop = 5,
    Xor = 6,
    Or = 7,
    Nor = 8,
    Equiv = 9,
    Inverted = 10,
    OrReverse = 11,
    CopyInverted = 12,
    OrInverted = 13,
    Nand = 14,
    Set = 15,
}

/// Texture Filter
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum TextureFilter {
    Nearest = 0,
    Linear = 1,
    NearestMipmapNearest = 4,
    LinearMipmapNearest = 5,
    NearestMipmapLinear = 6,
    LinearMipmapLinear = 7,
}

/// Texture Map Mode
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum TextureMapMode {
    TextureCoords = 0,
    TextureMatrix = 1,
    EnvironmentMap = 2,
}

/// Texture Level Mode
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum TextureLevelMode {
    Auto = 0,
    Const = 1,
    Slope = 2,
}

/// Texture Projection Map Mode
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum TextureProjectionMapMode {
    Position = 0,
    Uv = 1,
    NormalizedNormal = 2,
    Normal = 3,
}

/// Wrap Mode
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum GuTexWrapMode {
    /// The texture repeats after crossing the border
    Repeat = 0,

    /// Texture clamps at the border
    Clamp = 1,
}

/// Front Face Direction
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum FrontFaceDirection {
    Clockwise = 0,
    CounterClockwise = 1,
}

/// Test function for alpha test
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum AlphaFunc {
    Never = 0,
    Always,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

/// Test function for stencil test
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum StencilFunc {
    Never = 0,
    Always,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

/// Test function for color test
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum ColorFunc {
    Never = 0,
    Always,
    Equal,
    NotEqual,
}

/// Test function for depth test
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum DepthFunc {
    /// No pixels pass the depth-test
    Never = 0,
    /// All pixels pass the depth-test
    Always,
    /// Pixels that match the depth-test pass
    Equal,
    /// Pixels that doesn't match the depth-test pass
    NotEqual,
    /// Pixels that are less in depth passes
    Less,
    /// Pixels that are less or equal in depth passes
    LessOrEqual,
    /// Pixels that are greater in depth passes
    Greater,
    /// Pixels that are greater or equal passes
    GreaterOrEqual,
}

bitflags::bitflags! {
    /// Clear Buffer Mask
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub struct ClearBuffer: u32 {
        /// Clears the color buffer.
        const COLOR_BUFFER_BIT = 1;
        /// Clears the stencil buffer.
        const STENCIL_BUFFER_BIT = 2;
        /// Clears the depth buffer.
        const DEPTH_BUFFER_BIT = 4;
        /// Enables fast clearing. This divides the screen into 16 parts
        /// and clears them in parallel.
        const FAST_CLEAR_BIT = 16;
    }
}

/// Texture effect apply-modes.
///
/// Key for the apply-modes:
/// - `Cv`: Color value result
/// - `Ct`: Texture color
/// - `Cf`: Fragment color
/// - `Cc`: Constant color (specified by `sceGuTexEnvColor`)
///
/// The fields TCC_RGB and TCC_RGBA specify components that differ between
/// the two different component modes.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum TextureEffect {
    // TODO: Better documentation
    /// The texture is multiplied with the current diffuse fragment.
    ///
    /// `Cv=Ct*Cf TCC_RGB: Av=Af TCC_RGBA: Av=At*Af`
    Modulate = 0,
    /// `TCC_RGB: Cv=Ct,Av=Af TCC_RGBA: Cv=Cf*(1-At)+Ct*At Av=Af`
    Decal = 1,
    /// `Cv=(Cf*(1-Ct))+(Cc*Ct) TCC_RGB: Av=Af TCC_RGBA: Av=At*Af`
    Blend = 2,
    /// The texture replaces the fragment
    ///
    /// `Cv=Ct TCC_RGB: Av=Af TCC_RGBA: Av=At`
    Replace = 3,
    /// The texture is added on-top of the diffuse fragment
    ///
    /// `Cv=Cf+Ct TCC_RGB: Av=Af TCC_RGBA: Av=At*Af`
    Add = 4,
}

/// Texture color component-modes.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum TextureColorComponent {
    /// The texture alpha does not have any effect.
    Rgb = 0,
    /// The texture alpha is taken into account.
    Rgba = 1,
}

/// Mipmap Level
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum MipmapLevel {
    None = 0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    Level6,
    Level7,
}

/// Blending Operation
///
/// Keys for the blending operations:
///
/// - `Cs`: Source color
/// - `Cd`: Destination color
/// - `Bs`: Blend function for source fragment
/// - `Bd`: Blend function for destination fragment
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum BlendOp {
    /// `(Cs*Bs) + (Cd*Bd)`
    Add = 0,
    /// `(Cs*Bs) - (Cd*Bd)`
    Subtract = 1,
    /// `(Cd*Bd) - (Cs*Bs)`
    ReverseSubtract = 2,
    /// `Cs < Cd ? Cs : Cd`
    Min = 3,
    /// `Cs < Cd ? Cd : Cs`
    Max = 4,
    /// `|Cs-Cd|`
    Abs = 5,
}

/// Blending factor
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum BlendFactor {
    Color = 0,
    OneMinusColor = 1,
    SrcAlpha = 2,
    OneMinusSrcAlpha = 3,
    DstAlpha = 4,
    OneMinusDstAlpha = 5,
    // TODO: There are likely 4 missing values here.
    // What are 6, 7, 8, 9? This can probably be determined with some experimentation.
    // They may also be reserved values.
    /// Use the fixed values provided in `sceGuBlendFunc`.
    Fix = 10,
}

/// Stencil Operations
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum StencilOperation {
    /// Keeps the current value
    Keep = 0,
    /// Sets the stencil buffer value to zero
    Zero = 1,
    /// Sets the stencil buffer value to ref, as specified by `sceGuStencilFunc`
    Replace = 2,
    /// Increments the current stencil buffer value
    Invert = 3,
    /// Decrease the current stencil buffer value
    Incr = 4,
    /// Bitwise invert the current stencil buffer value
    Decr = 5,
}

bitflags::bitflags!(
    /// Light Components
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub struct LightComponent: i32 {
        const AMBIENT = 1;
        const DIFFUSE = 2;
        const SPECULAR = 4;

        // TODO: What is this?
        const UNKNOWN_LIGHT_COMPONENT = 8;
    }
);

/// Light modes
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum LightMode {
    SingleColor = 0,

    /// Separate specular colors are used to interpolate the specular component
    /// independently, so that it can be added to the fragment after the texture
    /// color.
    SeparateSpecularColor = 1,
}

/// Light Type
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum LightType {
    Directional = 0,
    Pointlight = 1,
    Spotlight = 2,
}

/// Contexts
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum GuContextType {
    Direct = 0,
    Call = 1,
    Send = 2,
}

/// List Queue Mode
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum GuQueueMode {
    /// Place list last in the queue, so it executes in-order
    Tail = 0,
    /// Place list first in queue so that it executes as soon as possible
    Head = 1,
}

/// Sync mode
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum GuSyncMode {
    /// Wait until the last sceGuFinish command is reached.
    Finish = 0,
    /// Wait until the last (?) signal is executed.
    Signal = 1,
    /// Wait until all commands currently in list are executed.
    Done = 2,
    /// Wait for the currently executed display list (`GuContextType::Direct`).
    List = 3,
    /// Wait for the last send list.
    Send = 4,
}

/// Sync Behavior
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum GuSyncBehavior {
    /// Wait for the GE list to be completed.
    Wait = 0,
    /// Just peek at the current state.
    NoWait = 1,
}

/// GU Callback ID
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum GuCallbackId {
    /// Called when `sceGuSignal` is used.
    Signal = 1,

    /// Called when display list is finished.
    Finish = 4,
}

/// Signal behavior
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum SignalBehavior {
    /// Stops display list execution until callback function finished.
    Suspend = 1,
    /// Do not stop display list execution during callback.
    Continue = 2,
}

/// Map 8-bit color channels into one 32-bit value.
#[inline]
pub const fn abgr(a: u8, b: u8, g: u8, r: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((a as u32) << 24)
}

/// Map 8-bit color channels into one 32-bit value.
#[inline]
pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    abgr(a, b, g, r)
}

/// Map 8-bit color channels into one 32-bit value.
#[inline]
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    argb(a, r, g, b)
}

#[inline]
/// Map floating point channels (0..1) into one 32-bit value
pub fn color(r: f32, g: f32, b: f32, a: f32) -> u32 {
    rgba(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    )
}

pub type GuCallback = Option<extern "C" fn(id: i32, arg: *mut c_void)>;
pub type GuSwapBuffersCallback =
    Option<extern "C" fn(display: *mut *mut c_void, render: *mut *mut c_void)>;
