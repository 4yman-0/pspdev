#![allow(static_mut_refs)]

use crate::sys::{
    self,
    display::DisplayPixelFormat,
    ge::{GeCommand, GeContext, GeListArgs, GeListState},
    kernel::SceUid,
};
use core::{ffi::c_void, mem, ptr::addr_of_mut, ptr::null_mut};
//use num_enum::TryFromPrimitive;

#[allow(clippy::approx_constant)]
pub const GU_PI: f32 = 3.141593;

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
    const fn num_weights(n: u32) -> i32 {
        (((n - 1) & 7) << 14) as i32
    }

    const fn num_vertices(n: u32) -> i32 {
        (((n - 1) & 7) << 18) as i32
    }
}

/// Texture pixel formats
// TODO: Better documentation
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Clone, Copy)]
#[repr(u32)]
// TODO: Should this be `ShadeMode` (no L)?
pub enum ShadingModel {
    Flat = 0,
    Smooth = 1,
}

/// Logical operation
#[derive(Clone, Copy)]
#[repr(u32)]
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
#[derive(Clone, Copy)]
#[repr(u32)]
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
#[derive(Clone, Copy)]
#[repr(u32)]
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

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum GuTexWrapMode {
    /// The texture repeats after crossing the border
    Repeat = 0,

    /// Texture clamps at the border
    Clamp = 1,
}

/// Front Face Direction
#[derive(Clone, Copy)]
#[repr(u32)]
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
    pub struct ClearFlags: u32 {
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
#[derive(Clone, Copy)]
#[repr(u32)]
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
#[derive(Clone, Copy)]
#[repr(u32)]
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
#[derive(Clone, Copy)]
#[repr(u32)]
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
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum LightMode {
    SingleColor = 0,

    /// Separate specular colors are used to interpolate the specular component
    /// independently, so that it can be added to the fragment after the texture
    /// color.
    SeparateSpecularColor = 1,
}

/// Light Type
#[derive(Clone, Copy)]
#[repr(u32)]
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
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum GuQueueMode {
    /// Place list last in the queue, so it executes in-order
    Tail = 0,
    /// Place list first in queue so that it executes as soon as possible
    Head = 1,
}

/// Sync mode
#[derive(Clone, Copy)]
#[repr(u32)]
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
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum GuSyncBehavior {
    /// Wait for the GE list to be completed.
    Wait = 0,
    /// Just peek at the current state.
    NoWait = 1,
}

/// GU Callback ID
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum GuCallbackId {
    /// Called when `sceGuSignal` is used.
    Signal = 1,

    /// Called when display list is finished.
    Finish = 4,
}

/// Signal behavior
#[derive(Clone, Copy)]
#[repr(u32)]
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

struct Settings {
    sig: GuCallback,
    fin: GuCallback,
    signal_history: [i16; 16],
    signal_offset: u32,
    kernel_event_flag: SceUid,
    ge_callback_id: i32,
    swap_buffers_callback: GuSwapBuffersCallback,
    swap_buffers_behaviour: crate::sys::DisplaySetBufSync,
}

struct GuDisplayList {
    start: *mut u32,
    current: *mut u32,
    parent_context: GuContextType,
}

struct GuContext {
    list: GuDisplayList,
}

struct GuDrawBuffer {
    pixel_size: DisplayPixelFormat,
    frame_width: i32,
    frame_buffer: *mut c_void,
    disp_buffer: *mut c_void,
    depth_buffer: *mut c_void,
    depth_width: i32,
    width: i32,
    height: i32,
}

static mut CONTEXTS: [GuContext; 3] = [
    GuContext {
        list: GuDisplayList {
            start: null_mut(),
            current: null_mut(),
            parent_context: GuContextType::Direct,
        },
    },
    GuContext {
        list: GuDisplayList {
            start: null_mut(),
            current: null_mut(),
            parent_context: GuContextType::Direct,
        },
    },
    GuContext {
        list: GuDisplayList {
            start: null_mut(),
            current: null_mut(),
            parent_context: GuContextType::Direct,
        },
    },
];

static mut GE_LIST_EXECUTED: [i32; 2] = [0, 0];
static mut GE_EDRAM_ADDRESS: *mut c_void = null_mut();

static mut SETTINGS: Settings = Settings {
    sig: None,
    fin: None,
    signal_history: [0; 16],
    signal_offset: 0,

    // Invalid UID until initialized.
    kernel_event_flag: SceUid(-1),

    ge_callback_id: 0,
    swap_buffers_behaviour: crate::sys::DisplaySetBufSync::Immediate,
    swap_buffers_callback: None,
};

static mut LIST: *mut GuDisplayList = null_mut();
static mut CURR_CONTEXT: GuContextType = GuContextType::Direct;
static mut INIT: i32 = 0;
static mut DISPLAY_ON: bool = false;
static mut CALL_MODE: i32 = 0;

static mut DRAW_BUFFER: GuDrawBuffer = GuDrawBuffer {
    depth_buffer: null_mut(),
    frame_buffer: null_mut(),
    disp_buffer: null_mut(),
    width: 0,
    height: 0,
    depth_width: 0,
    frame_width: 0,
    pixel_size: DisplayPixelFormat::Psm5650,
};

static mut OBJECT_STACK_DEPTH: i32 = 0;

#[inline]
unsafe fn send_command_i(cmd: GeCommand, argument: i32) {
    (*(*LIST).current) = ((cmd as u32) << 24) | (argument as u32 & 0xffffff);
    (*LIST).current = (*LIST).current.add(1);
}

#[inline]
unsafe fn send_command_f(cmd: GeCommand, argument: f32) {
    send_command_i(cmd, (argument.to_bits() >> 8) as i32);
}

unsafe fn command_stall() {
    if let (GuContextType::Direct, 0) = (CURR_CONTEXT, OBJECT_STACK_DEPTH) {
        crate::sys::sceGeListUpdateStallAddr(
            GE_LIST_EXECUTED[0],
            (*LIST).current as *mut c_void,
        );
    }
}

#[inline]
unsafe fn send_command_i_stall(cmd: GeCommand, argument: i32) {
    send_command_i(cmd, argument);
    command_stall();
}

unsafe fn draw_region(x: i32, y: i32, width: i32, height: i32) {
    send_command_i(GeCommand::Region1, (y << 10) | x);
    send_command_i(
        GeCommand::Region2,
        (((y + height) - 1) << 10) | ((x + width) - 1),
    );
}

unsafe fn reset_values() {
    INIT = 0;
    OBJECT_STACK_DEPTH = 0;
    DISPLAY_ON = false;
    CALL_MODE = 0;
    DRAW_BUFFER.pixel_size = DisplayPixelFormat::Psm5551;
    DRAW_BUFFER.frame_width = 0;
    DRAW_BUFFER.frame_buffer = null_mut();
    DRAW_BUFFER.disp_buffer = null_mut();
    DRAW_BUFFER.depth_buffer = null_mut();
    DRAW_BUFFER.depth_width = 0;
    DRAW_BUFFER.width = 480;
    DRAW_BUFFER.height = 272;

    SETTINGS.sig = None;
    SETTINGS.fin = None;
}

extern "C" fn callback_sig(id: i32, arg: *mut c_void) {
    let settings = arg as *mut Settings;

    unsafe {
        let idx = ((*settings).signal_offset & 15) as usize;
        (*settings).signal_history[idx] = (id & 0xffff) as i16;
        (*settings).signal_offset += 1;

        if (*settings).sig.is_some() {
            // Convert Option<fn(i32, *mut c_void)> -> fn(i32)
            // This is fine because we are transmuting a nullable function
            // pointer to another function pointer. The requirement here is that
            // it must not be null.
            let f: extern "C" fn(i32) = mem::transmute((*settings).sig);

            f(id & 0xffff);
        }

        crate::sys::sceKernelSetEventFlag((*settings).kernel_event_flag, 1);
    }
}

extern "C" fn callback_fin(id: i32, arg: *mut c_void) {
    unsafe {
        let settings = arg as *mut Settings;

        if let Some(fin) = (*settings).fin {
            // Convert Option<fn(i32, *mut c_void)> -> fn(i32)
            // This is fine because we are transmuting a nullable function
            // pointer to another function pointer. The requirement here is that
            // it must not be null.
            let f: extern "C" fn(i32) = core::mem::transmute(fin);

            f(id & 0xffff)
        }
    }
}

/// Set depth buffer parameters
///
/// # Parameters
///
/// - `zbp`: VRAM pointer where the depth buffer should start
/// - `zbw`: The width of the depth buffer (block-aligned)
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuDepthBuffer(zbp: *mut c_void, zbw: i32) {
    DRAW_BUFFER.depth_buffer = zbp;

    if DRAW_BUFFER.depth_width == 0 || DRAW_BUFFER.depth_width != zbw {
        DRAW_BUFFER.depth_width = zbw;
    }

    send_command_i(GeCommand::ZBufPtr, zbp as i32 & 0xffffff);
    send_command_i(
        GeCommand::ZBufWidth,
        (((zbp as u32 & 0xff000000) >> 8) | zbw as u32) as i32,
    );
}

/// Set display buffer parameters
///
/// # Parameters
///
/// - `width`: Width of the display buffer in pixels
/// - `height`: Width of the display buffer in pixels
/// - `dispbp`: VRAM pointer to where the display-buffer starts
/// - `dispbw`: Display buffer width (block aligned)
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuDispBuffer(
    width: i32,
    height: i32,
    dispbp: *mut c_void,
    dispbw: i32,
) {
    use crate::sys::DisplaySetBufSync;

    DRAW_BUFFER.width = width;
    DRAW_BUFFER.height = height;
    DRAW_BUFFER.disp_buffer = dispbp;

    if DRAW_BUFFER.frame_width == 0 || DRAW_BUFFER.frame_width != dispbw {
        DRAW_BUFFER.frame_width = dispbw;
    }

    draw_region(0, 0, DRAW_BUFFER.width, DRAW_BUFFER.height);

    crate::sys::sceDisplaySetMode(
        crate::sys::DisplayMode::Lcd,
        DRAW_BUFFER.width as usize,
        DRAW_BUFFER.height as usize,
    );

    if DISPLAY_ON {
        crate::sys::sceDisplaySetFrameBuf(
            (GE_EDRAM_ADDRESS as *mut u8).add(DRAW_BUFFER.disp_buffer as usize),
            dispbw as usize,
            DRAW_BUFFER.pixel_size,
            DisplaySetBufSync::NextFrame,
        );
    }
}

/// Set draw buffer parameters (and store in context for buffer-swap)
///
/// # Parameters
///
/// - `psm`: Pixel format to use for rendering (and display)
/// - `fbp`: VRAM pointer to where the draw buffer starts
/// - `fbw`: Frame buffer width (block aligned)
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuDrawBuffer(
    psm: DisplayPixelFormat,
    fbp: *mut c_void,
    fbw: i32,
) {
    DRAW_BUFFER.pixel_size = psm;
    DRAW_BUFFER.frame_width = fbw;
    DRAW_BUFFER.frame_buffer = fbp;

    if DRAW_BUFFER.depth_buffer.is_null() && DRAW_BUFFER.height != 0 {
        DRAW_BUFFER.depth_buffer = (fbp as u32
            + (((DRAW_BUFFER.height * fbw) as u32) << 2u32))
            as *mut c_void;
    }

    if DRAW_BUFFER.depth_width == 0 {
        DRAW_BUFFER.depth_width = fbw;
    }

    send_command_i(GeCommand::FramebufPixFormat, psm as i32);
    send_command_i(
        GeCommand::FrameBufPtr,
        DRAW_BUFFER.frame_buffer as i32 & 0xffffff,
    );
    send_command_i(
        GeCommand::FrameBufWidth,
        ((DRAW_BUFFER.frame_buffer as u32 & 0xff000000) >> 8) as i32
            | DRAW_BUFFER.frame_width,
    );
    send_command_i(
        GeCommand::ZBufPtr,
        DRAW_BUFFER.depth_buffer as i32 & 0xffffff,
    );
    send_command_i(
        GeCommand::ZBufWidth,
        ((DRAW_BUFFER.depth_buffer as u32 & 0xff000000) >> 8) as i32
            | DRAW_BUFFER.depth_width,
    );
}

/// Set draw buffer directly, not storing parameters in the context
///
/// # Parameters
///
/// - `psm`: Pixel format to use for rendering
/// - `fbp`: VRAM pointer to where the draw buffer starts
/// - `fbw`: Frame buffer width (block aligned)
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuDrawBufferList(
    psm: DisplayPixelFormat,
    fbp: *mut c_void,
    fbw: i32,
) {
    send_command_i(GeCommand::FramebufPixFormat, psm as i32);
    send_command_i(GeCommand::FrameBufPtr, fbp as i32 & 0xffffff);
    send_command_i(
        GeCommand::FrameBufWidth,
        ((fbp as u32 & 0xff000000) >> 8) as i32 | fbw,
    );
}

/// Initalize the GU system
///
/// This function MUST be called as the first function, otherwise state is undetermined.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuInit() {
    const INIT_COMMANDS: [GeCommand; 223] = [
        GeCommand::Vaddr,
        GeCommand::Iaddr,
        GeCommand::Base,
        GeCommand::VertexType,
        GeCommand::OffsetAddr,
        GeCommand::Region1,
        GeCommand::Region2,
        GeCommand::LightingEnable,
        GeCommand::LightEnable0,
        GeCommand::LightEnable1,
        GeCommand::LightEnable2,
        GeCommand::LightEnable3,
        GeCommand::DepthClampEnable,
        GeCommand::CullFaceEnable,
        GeCommand::TextureMapEnable,
        GeCommand::FogEnable,
        GeCommand::DitherEnable,
        GeCommand::AlphaBlendEnable,
        GeCommand::AlphaTestEnable,
        GeCommand::ZTestEnable,
        GeCommand::StencilTestEnable,
        GeCommand::AntiAliasEnable,
        GeCommand::PatchCullEnable,
        GeCommand::ColorTestEnable,
        GeCommand::LogicOpEnable,
        GeCommand::BoneMatrixNumber,
        GeCommand::BoneMatrixData,
        GeCommand::MorphWeight0,
        GeCommand::MorphWeight1,
        GeCommand::MorphWeight2,
        GeCommand::MorphWeight3,
        GeCommand::MorphWeight4,
        GeCommand::MorphWeight5,
        GeCommand::MorphWeight6,
        GeCommand::MorphWeight7,
        GeCommand::PatchDivision,
        GeCommand::PatchPrimitive,
        GeCommand::PatchFacing,
        GeCommand::WorldMatrixNumber,
        GeCommand::WorldMatrixData,
        GeCommand::ViewMatrixNumber,
        GeCommand::ViewMatrixData,
        GeCommand::ProjMatrixNumber,
        GeCommand::ProjMatrixData,
        GeCommand::TGenMatrixNumber,
        GeCommand::TGenMatrixData,
        GeCommand::ViewportXScale,
        GeCommand::ViewportYScale,
        GeCommand::ViewportZScale,
        GeCommand::ViewportXCenter,
        GeCommand::ViewportYCenter,
        GeCommand::ViewportZCenter,
        GeCommand::TexScaleU,
        GeCommand::TexScaleV,
        GeCommand::TexOffsetU,
        GeCommand::TexOffsetV,
        GeCommand::OffsetX,
        GeCommand::OffsetY,
        GeCommand::ShadeMode,
        GeCommand::ReverseNormal,
        GeCommand::MaterialUpdate,
        GeCommand::MaterialEmissive,
        GeCommand::MaterialAmbient,
        GeCommand::MaterialDiffuse,
        GeCommand::MaterialSpecular,
        GeCommand::MaterialAlpha,
        GeCommand::MaterialSpecularCoef,
        GeCommand::AmbientColor,
        GeCommand::AmbientAlpha,
        GeCommand::LightMode,
        GeCommand::LightType0,
        GeCommand::LightType1,
        GeCommand::LightType2,
        GeCommand::LightType3,
        GeCommand::Light0X,
        GeCommand::Light0Y,
        GeCommand::Light0Z,
        GeCommand::Light1X,
        GeCommand::Light1Y,
        GeCommand::Light1Z,
        GeCommand::Light2X,
        GeCommand::Light2Y,
        GeCommand::Light2Z,
        GeCommand::Light3X,
        GeCommand::Light3Y,
        GeCommand::Light3Z,
        GeCommand::Light0DirectionX,
        GeCommand::Light0DirectionY,
        GeCommand::Light0DirectionZ,
        GeCommand::Light1DirectionX,
        GeCommand::Light1DirectionY,
        GeCommand::Light1DirectionZ,
        GeCommand::Light2DirectionX,
        GeCommand::Light2DirectionY,
        GeCommand::Light2DirectionZ,
        GeCommand::Light3DirectionX,
        GeCommand::Light3DirectionY,
        GeCommand::Light3DirectionZ,
        GeCommand::Light0ConstantAtten,
        GeCommand::Light0LinearAtten,
        GeCommand::Light0QuadtraticAtten,
        GeCommand::Light1ConstantAtten,
        GeCommand::Light1LinearAtten,
        GeCommand::Light1QuadtraticAtten,
        GeCommand::Light2ConstantAtten,
        GeCommand::Light2LinearAtten,
        GeCommand::Light2QuadtraticAtten,
        GeCommand::Light3ConstantAtten,
        GeCommand::Light3LinearAtten,
        GeCommand::Light3QuadtraticAtten,
        GeCommand::Light0ExponentAtten,
        GeCommand::Light1ExponentAtten,
        GeCommand::Light2ExponentAtten,
        GeCommand::Light3ExponentAtten,
        GeCommand::Light0CutoffAtten,
        GeCommand::Light1CutoffAtten,
        GeCommand::Light2CutoffAtten,
        GeCommand::Light3CutoffAtten,
        GeCommand::Light0Ambient,
        GeCommand::Light0Diffuse,
        GeCommand::Light0Specular,
        GeCommand::Light1Ambient,
        GeCommand::Light1Diffuse,
        GeCommand::Light1Specular,
        GeCommand::Light2Ambient,
        GeCommand::Light2Diffuse,
        GeCommand::Light2Specular,
        GeCommand::Light3Ambient,
        GeCommand::Light3Diffuse,
        GeCommand::Light3Specular,
        GeCommand::Cull,
        GeCommand::FrameBufPtr,
        GeCommand::FrameBufWidth,
        GeCommand::ZBufPtr,
        GeCommand::ZBufWidth,
        GeCommand::TexAddr0,
        GeCommand::TexAddr1,
        GeCommand::TexAddr2,
        GeCommand::TexAddr3,
        GeCommand::TexAddr4,
        GeCommand::TexAddr5,
        GeCommand::TexAddr6,
        GeCommand::TexAddr7,
        GeCommand::TexBufWidth0,
        GeCommand::TexBufWidth1,
        GeCommand::TexBufWidth2,
        GeCommand::TexBufWidth3,
        GeCommand::TexBufWidth4,
        GeCommand::TexBufWidth5,
        GeCommand::TexBufWidth6,
        GeCommand::TexBufWidth7,
        GeCommand::ClutAddr,
        GeCommand::ClutAddrUpper,
        GeCommand::TransferSrc,
        GeCommand::TransferSrcW,
        GeCommand::TransferDst,
        GeCommand::TransferDstW,
        GeCommand::TexSize0,
        GeCommand::TexSize1,
        GeCommand::TexSize2,
        GeCommand::TexSize3,
        GeCommand::TexSize4,
        GeCommand::TexSize5,
        GeCommand::TexSize6,
        GeCommand::TexSize7,
        GeCommand::TexMapMode,
        GeCommand::TexShadeLs,
        GeCommand::TexMode,
        GeCommand::TexFormat,
        GeCommand::LoadClut,
        GeCommand::ClutFormat,
        GeCommand::TexFilter,
        GeCommand::TexWrap,
        GeCommand::TexLevel,
        GeCommand::TexFunc,
        GeCommand::TexEnvColor,
        GeCommand::TexFlush,
        GeCommand::TexSync,
        GeCommand::Fog1,
        GeCommand::Fog2,
        GeCommand::FogColor,
        GeCommand::TexLodSlope,
        GeCommand::FramebufPixFormat,
        GeCommand::ClearMode,
        GeCommand::Scissor1,
        GeCommand::Scissor2,
        GeCommand::MinZ,
        GeCommand::MaxZ,
        GeCommand::ColorTest,
        GeCommand::ColorRef,
        GeCommand::ColorTestmask,
        GeCommand::AlphaTest,
        GeCommand::StencilTest,
        GeCommand::StencilOp,
        GeCommand::ZTest,
        GeCommand::BlendMode,
        GeCommand::BlendFixedA,
        GeCommand::BlendFixedB,
        GeCommand::Dith0,
        GeCommand::Dith1,
        GeCommand::Dith2,
        GeCommand::Dith3,
        GeCommand::LogicOp,
        GeCommand::ZWriteDisable,
        GeCommand::MaskRgb,
        GeCommand::MaskAlpha,
        GeCommand::TransferSrcPos,
        GeCommand::TransferDstPos,
        GeCommand::TransferSize,
        GeCommand::Vscx,
        GeCommand::Vscy,
        GeCommand::Vscz,
        GeCommand::Vtcs,
        GeCommand::Vtct,
        GeCommand::Vtcq,
        GeCommand::Vcv,
        GeCommand::Vap,
        GeCommand::Vfc,
        GeCommand::Vscv,
        GeCommand::Finish,
        GeCommand::End,
        GeCommand::Nop,
        GeCommand::Nop,
    ];

    static INIT_LIST: crate::Align16<[u32; 223]> = crate::Align16({
        let mut out = [0; 223];

        let mut i = 0;
        while i < 223 {
            out[i] = (INIT_COMMANDS[i] as u32) << 24;
            i += 1;
        }

        out
    });

    let mut callback = crate::sys::GeCallbackData {
        signal_func: Some(callback_sig),
        signal_arg: addr_of_mut!(SETTINGS).cast::<c_void>(),
        finish_func: Some(callback_fin),
        finish_arg: addr_of_mut!(SETTINGS).cast::<c_void>(),
    };

    SETTINGS.ge_callback_id = crate::sys::sceGeSetCallback(&mut callback);
    SETTINGS.swap_buffers_callback = None;
    SETTINGS.swap_buffers_behaviour =
        super::display::DisplaySetBufSync::Immediate;

    GE_EDRAM_ADDRESS = sys::sceGeEdramGetAddr().cast::<c_void>();

    GE_LIST_EXECUTED[0] = sys::sceGeListEnQueue(
        (&INIT_LIST as *const _ as u32 & 0x1fffffff) as *const _,
        core::ptr::null_mut(),
        SETTINGS.ge_callback_id,
        core::ptr::null_mut(),
    );

    reset_values();

    SETTINGS.kernel_event_flag = super::kernel::sceKernelCreateEventFlag(
        b"SceGuSignal\0" as *const u8,
        super::kernel::EventFlagAttributes::WAIT_MULTIPLE,
        3,
        null_mut(),
    );

    sys::sceGeListSync(GE_LIST_EXECUTED[0], 0);
}

/// Shutdown the GU system
///
/// Called when GU is no longer needed
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuTerm() {
    sys::sceKernelDeleteEventFlag(SETTINGS.kernel_event_flag);
    sys::sceGeUnsetCallback(SETTINGS.ge_callback_id);
}

/// Send raw float command to the GE
///
/// The argument is converted into a 24-bit float before transfer.
///
/// # Parameters
///
/// - `cmd`: Which command to send
/// - `argument`: Argument to pass along
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuSendCommandf(cmd: GeCommand, argument: f32) {
    send_command_f(cmd, argument);
}

/// Send raw command to the GE
///
/// Only the 24 lower bits of the argument are passed along.
///
/// # Parameters
///
/// - `cmd`: Which command to send
/// - `argument`: Argument to pass along
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuSendCommandi(cmd: GeCommand, argument: i32) {
    send_command_i(cmd, argument);
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuCommandStall() {
    command_stall();
}

/// Start filling a new display-context
///
/// The previous context-type is stored so that it can be restored at `sceGuFinish`.
///
/// # Parameters
///
/// - `cid`: Context Type
/// - `list`: Pointer to display-list (16 byte aligned)
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuStart(
    context_type: GuContextType,
    list: *mut c_void,
) {
    let context = &mut CONTEXTS[context_type as usize];
    let local_list = ((list as u32) | 0x4000_0000) as *mut u32;

    // setup display list
    context.list.start = local_list;
    context.list.current = local_list;
    context.list.parent_context = CURR_CONTEXT;
    LIST = &mut context.list;

    // store current context
    CURR_CONTEXT = context_type;

    if let GuContextType::Direct = context_type {
        GE_LIST_EXECUTED[0] = crate::sys::sceGeListEnQueue(
            local_list as *mut c_void,
            local_list as *mut c_void,
            SETTINGS.ge_callback_id,
            core::ptr::null_mut(),
        );

        SETTINGS.signal_offset = 0;
    }

    if INIT == 0 {
        INIT = 1;
    }
}

/// Finish current display list and go back to the parent context
///
/// If the context is `Direct`, the stall-address is updated so that the entire
/// list will execute. Otherwise, only the terminating action is written to the
/// list, depending on the context type.
///
/// The finish-callback will get a zero as argument when using this function.
///
/// This also restores control back to whatever context that was active prior to
/// this call.
///
/// # Return Value
///
/// Size of finished display list
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuFinish() -> i32 {
    match CURR_CONTEXT {
        GuContextType::Direct | GuContextType::Send => {
            send_command_i(GeCommand::Finish, 0);
            send_command_i_stall(GeCommand::End, 0);
        }

        GuContextType::Call => {
            if CALL_MODE == 1 {
                send_command_i(GeCommand::Signal, 0x120000);
                send_command_i(GeCommand::End, 0);
                send_command_i_stall(GeCommand::Nop, 0);
            } else {
                send_command_i(GeCommand::Ret, 0);
            }
        }
    }

    let size = ((*LIST).current as usize) - ((*LIST).start as usize);

    // Go to parent list
    CURR_CONTEXT = (*LIST).parent_context;
    LIST = &mut CONTEXTS[CURR_CONTEXT as usize].list;
    size as i32
}

/// Check how large the current display list is
///
/// # Return Value
///
/// The size of the current display list
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuCheckList() -> i32 {
    (*LIST).current.sub((*LIST).start as usize) as i32
}

/// Send a list to the GE directly
///
/// # Parameters
///
/// - `mode`: Whether to place the list first or last in queue
/// - `list`: List to send
/// - `context`: Temporary storage for the GE context
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuSendList(
    mode: GuQueueMode,
    list: *const c_void,
    context: *mut GeContext,
) {
    SETTINGS.signal_offset = 0;

    let mut args = GeListArgs {
        size: 8,
        context,
        ..<_>::default()
    };

    let callback = SETTINGS.ge_callback_id;

    let list_id = match mode {
        GuQueueMode::Head => crate::sys::sceGeListEnQueueHead(
            list,
            null_mut(),
            callback,
            &mut args,
        ),

        GuQueueMode::Tail => {
            crate::sys::sceGeListEnQueue(list, null_mut(), callback, &mut args)
        }
    };

    GE_LIST_EXECUTED[1] = list_id;
}

/// Wait until display list has finished executing
///
/// # Parameters
///
/// - `mode`: What to wait for, one of `GuSyncMode`
/// - `behavior`: How to sync, one of `GuSyncBehavior`
///
/// # Return Value
///
/// Unknown at this time. GeListState?
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuSync(
    mode: GuSyncMode,
    behavior: GuSyncBehavior,
) -> GeListState {
    match mode {
        GuSyncMode::Finish => crate::sys::sceGeDrawSync(behavior as i32),
        GuSyncMode::List => {
            crate::sys::sceGeListSync(GE_LIST_EXECUTED[0], behavior as i32)
        }
        GuSyncMode::Send => {
            crate::sys::sceGeListSync(GE_LIST_EXECUTED[1], behavior as i32)
        }
        _ => GeListState::Done,
    }
}

// TODO: Maybe add examples in documentation?
/// Image transfer using the GE
///
/// # Note
///
/// Data must be aligned to 1 quad word (16 bytes)
///
/// # Parameters
///
/// - `psm`: Pixel format for buffer
/// - `sx`: Source X
/// - `sy`: Source Y
/// - `width`: Image width
/// - `height`: Image height
/// - `srcw`: Source buffer width (block aligned)
/// - `src`: Source pointer
/// - `dx`: Destination X
/// - `dy`: Destination Y
/// - `destw`: Destination buffer width (block aligned)
/// - `dest`: Destination pointer
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sceGuCopyImage(
    psm: DisplayPixelFormat,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    srcw: i32,
    src: *mut c_void,
    dx: i32,
    dy: i32,
    destw: i32,
    dest: *mut c_void,
) {
    send_command_i(GeCommand::TransferSrc, (src as i32) & 0xffffff);
    send_command_i(
        GeCommand::TransferSrcW,
        (((src as u32) & 0xff000000) >> 8) as i32 | srcw,
    );
    send_command_i(GeCommand::TransferSrcPos, (sy << 10) | sx);
    send_command_i(GeCommand::TransferDst, (dest as i32) & 0xffffff);
    send_command_i(
        GeCommand::TransferDstW,
        (((dest as u32) & 0xff000000) >> 8) as i32 | destw,
    );
    send_command_i(GeCommand::TransferDstPos, (dy << 10) | dx);
    send_command_i(GeCommand::TransferSize, ((height - 1) << 10) | (width - 1));

    let is_32_bit_texel = if let DisplayPixelFormat::Psm8888 = psm {
        1
    } else {
        0
    };

    send_command_i(GeCommand::TransferStart, is_32_bit_texel);
}
