use bitflags::bitflags;
use vulkanalia::vk::Flags;

pub const RHI_SUBPASS_EXTERNAL: u32 = !0;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIAttachmentLoadOp(i32);
impl RHIAttachmentLoadOp {
    pub const LOAD: Self = Self(0);
    pub const CLEAR: Self = Self(1);
    pub const DONT_CARE: Self = Self(2);
    pub const NONE: Self = Self(1000400000);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIAttachmentStoreOp(i32);
impl RHIAttachmentStoreOp {
    pub const STORE: Self = Self(0);
    pub const DONT_CARE: Self = Self(1);
    pub const NONE: Self = Self(1000301000);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIBlendFactor(i32);
impl RHIBlendFactor {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const SRC_COLOR: Self = Self(2);
    pub const ONE_MINUS_SRC_COLOR: Self = Self(3);
    pub const DST_COLOR: Self = Self(4);
    pub const ONE_MINUS_DST_COLOR: Self = Self(5);
    pub const SRC_ALPHA: Self = Self(6);
    pub const ONE_MINUS_SRC_ALPHA: Self = Self(7);
    pub const DST_ALPHA: Self = Self(8);
    pub const ONE_MINUS_DST_ALPHA: Self = Self(9);
    pub const CONSTANT_COLOR: Self = Self(10);
    pub const ONE_MINUS_CONSTANT_COLOR: Self = Self(11);
    pub const CONSTANT_ALPHA: Self = Self(12);
    pub const ONE_MINUS_CONSTANT_ALPHA: Self = Self(13);
    pub const SRC_ALPHA_SATURATE: Self = Self(14);
    pub const SRC1_COLOR: Self = Self(15);
    pub const ONE_MINUS_SRC1_COLOR: Self = Self(16);
    pub const SRC1_ALPHA: Self = Self(17);
    pub const ONE_MINUS_SRC1_ALPHA: Self = Self(18);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIBlendOp(i32);
impl RHIBlendOp {
    pub const ADD: Self = Self(0);
    pub const SUBTRACT: Self = Self(1);
    pub const REVERSE_SUBTRACT: Self = Self(2);
    pub const MIN: Self = Self(3);
    pub const MAX: Self = Self(4);
    pub const ZERO_EXT: Self = Self(1000148000);
    pub const SRC_EXT: Self = Self(1000148001);
    pub const DST_EXT: Self = Self(1000148002);
    pub const SRC_OVER_EXT: Self = Self(1000148003);
    pub const DST_OVER_EXT: Self = Self(1000148004);
    pub const SRC_IN_EXT: Self = Self(1000148005);
    pub const DST_IN_EXT: Self = Self(1000148006);
    pub const SRC_OUT_EXT: Self = Self(1000148007);
    pub const DST_OUT_EXT: Self = Self(1000148008);
    pub const SRC_ATOP_EXT: Self = Self(1000148009);
    pub const DST_ATOP_EXT: Self = Self(1000148010);
    pub const XOR_EXT: Self = Self(1000148011);
    pub const MULTIPLY_EXT: Self = Self(1000148012);
    pub const SCREEN_EXT: Self = Self(1000148013);
    pub const OVERLAY_EXT: Self = Self(1000148014);
    pub const DARKEN_EXT: Self = Self(1000148015);
    pub const LIGHTEN_EXT: Self = Self(1000148016);
    pub const COLORDODGE_EXT: Self = Self(1000148017);
    pub const COLORBURN_EXT: Self = Self(1000148018);
    pub const HARDLIGHT_EXT: Self = Self(1000148019);
    pub const SOFTLIGHT_EXT: Self = Self(1000148020);
    pub const DIFFERENCE_EXT: Self = Self(1000148021);
    pub const EXCLUSION_EXT: Self = Self(1000148022);
    pub const INVERT_EXT: Self = Self(1000148023);
    pub const INVERT_RGB_EXT: Self = Self(1000148024);
    pub const LINEARDODGE_EXT: Self = Self(1000148025);
    pub const LINEARBURN_EXT: Self = Self(1000148026);
    pub const VIVIDLIGHT_EXT: Self = Self(1000148027);
    pub const LINEARLIGHT_EXT: Self = Self(1000148028);
    pub const PINLIGHT_EXT: Self = Self(1000148029);
    pub const HARDMIX_EXT: Self = Self(1000148030);
    pub const HSL_HUE_EXT: Self = Self(1000148031);
    pub const HSL_SATURATION_EXT: Self = Self(1000148032);
    pub const HSL_COLOR_EXT: Self = Self(1000148033);
    pub const HSL_LUMINOSITY_EXT: Self = Self(1000148034);
    pub const PLUS_EXT: Self = Self(1000148035);
    pub const PLUS_CLAMPED_EXT: Self = Self(1000148036);
    pub const PLUS_CLAMPED_ALPHA_EXT: Self = Self(1000148037);
    pub const PLUS_DARKER_EXT: Self = Self(1000148038);
    pub const MINUS_EXT: Self = Self(1000148039);
    pub const MINUS_CLAMPED_EXT: Self = Self(1000148040);
    pub const CONTRAST_EXT: Self = Self(1000148041);
    pub const INVERT_OVG_EXT: Self = Self(1000148042);
    pub const RED_EXT: Self = Self(1000148043);
    pub const GREEN_EXT: Self = Self(1000148044);
    pub const BLUE_EXT: Self = Self(1000148045);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHICompareOp(i32);
impl RHICompareOp {
    pub const NEVER: Self = Self(0);
    pub const LESS: Self = Self(1);
    pub const EQUAL: Self = Self(2);
    pub const LESS_OR_EQUAL: Self = Self(3);
    pub const GREATER: Self = Self(4);
    pub const NOT_EQUAL: Self = Self(5);
    pub const GREATER_OR_EQUAL: Self = Self(6);
    pub const ALWAYS: Self = Self(7);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

pub enum RHIDefaultSamplerType{
    Linear,
    Nearest,
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIDescriptorType(i32);
impl RHIDescriptorType {
    pub const SAMPLER: Self = Self(0);
    pub const COMBINED_IMAGE_SAMPLER: Self = Self(1);
    pub const SAMPLED_IMAGE: Self = Self(2);
    pub const STORAGE_IMAGE: Self = Self(3);
    pub const UNIFORM_TEXEL_BUFFER: Self = Self(4);
    pub const STORAGE_TEXEL_BUFFER: Self = Self(5);
    pub const UNIFORM_BUFFER: Self = Self(6);
    pub const STORAGE_BUFFER: Self = Self(7);
    pub const UNIFORM_BUFFER_DYNAMIC: Self = Self(8);
    pub const STORAGE_BUFFER_DYNAMIC: Self = Self(9);
    pub const INPUT_ATTACHMENT: Self = Self(10);
    pub const INLINE_UNIFORM_BLOCK_EXT: Self = Self(1000138000);
    pub const ACCELERATION_STRUCTURE_KHR: Self = Self(1000150000);
    pub const ACCELERATION_STRUCTURE_NV: Self = Self(1000165000);
    pub const MUTABLE_VALVE: Self = Self(1000351000);
    pub const MAX_ENUM: Self = Self(0x7FFFFFFF);

    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIDynamicState(i32);
impl RHIDynamicState {
    pub const VIEWPORT: Self = Self(0);
    pub const SCISSOR: Self = Self(1);
    pub const LINE_WIDTH: Self = Self(2);
    pub const DEPTH_BIAS: Self = Self(3);
    pub const BLEND_CONSTANTS: Self = Self(4);
    pub const DEPTH_BOUNDS: Self = Self(5);
    pub const STENCIL_COMPARE_MASK: Self = Self(6);
    pub const STENCIL_WRITE_MASK: Self = Self(7);
    pub const STENCIL_REFERENCE: Self = Self(8);
    pub const CULL_MODE: Self = Self(1000267000);
    pub const FRONT_FACE: Self = Self(1000267001);
    pub const PRIMITIVE_TOPOLOGY: Self = Self(1000267002);
    pub const VIEWPORT_WITH_COUNT: Self = Self(1000267003);
    pub const SCISSOR_WITH_COUNT: Self = Self(1000267004);
    pub const VERTEX_INPUT_BINDING_STRIDE: Self = Self(1000267005);
    pub const DEPTH_TEST_ENABLE: Self = Self(1000267006);
    pub const DEPTH_WRITE_ENABLE: Self = Self(1000267007);
    pub const DEPTH_COMPARE_OP: Self = Self(1000267008);
    pub const DEPTH_BOUNDS_TEST_ENABLE: Self = Self(1000267009);
    pub const STENCIL_TEST_ENABLE: Self = Self(1000267010);
    pub const STENCIL_OP: Self = Self(1000267011);
    pub const RASTERIZER_DISCARD_ENABLE: Self = Self(1000377001);
    pub const DEPTH_BIAS_ENABLE: Self = Self(1000377002);
    pub const PRIMITIVE_RESTART_ENABLE: Self = Self(1000377004);
    pub const LINE_STIPPLE: Self = Self(1000259000);
    pub const VIEWPORT_W_SCALING_NV: Self = Self(1000087000);
    pub const DISCARD_RECTANGLE_EXT: Self = Self(1000099000);
    pub const DISCARD_RECTANGLE_ENABLE_EXT: Self = Self(1000099001);
    pub const DISCARD_RECTANGLE_MODE_EXT: Self = Self(1000099002);
    pub const SAMPLE_LOCATIONS_EXT: Self = Self(1000143000);
    pub const RAY_TRACING_PIPELINE_STACK_SIZE_KHR: Self = Self(1000347000);
    pub const VIEWPORT_SHADING_RATE_PALETTE_NV: Self = Self(1000164004);
    pub const VIEWPORT_COARSE_SAMPLE_ORDER_NV: Self = Self(1000164006);
    pub const EXCLUSIVE_SCISSOR_ENABLE_NV: Self = Self(1000205000);
    pub const EXCLUSIVE_SCISSOR_NV: Self = Self(1000205001);
    pub const FRAGMENT_SHADING_RATE_KHR: Self = Self(1000226000);
    pub const VERTEX_INPUT_EXT: Self = Self(1000352000);
    pub const PATCH_CONTROL_POINTS_EXT: Self = Self(1000377000);
    pub const LOGIC_OP_EXT: Self = Self(1000377003);
    pub const COLOR_WRITE_ENABLE_EXT: Self = Self(1000381000);
    pub const DEPTH_CLAMP_ENABLE_EXT: Self = Self(1000455003);
    pub const POLYGON_MODE_EXT: Self = Self(1000455004);
    pub const RASTERIZATION_SAMPLES_EXT: Self = Self(1000455005);
    pub const SAMPLE_MASK_EXT: Self = Self(1000455006);
    pub const ALPHA_TO_COVERAGE_ENABLE_EXT: Self = Self(1000455007);
    pub const ALPHA_TO_ONE_ENABLE_EXT: Self = Self(1000455008);
    pub const LOGIC_OP_ENABLE_EXT: Self = Self(1000455009);
    pub const COLOR_BLEND_ENABLE_EXT: Self = Self(1000455010);
    pub const COLOR_BLEND_EQUATION_EXT: Self = Self(1000455011);
    pub const COLOR_WRITE_MASK_EXT: Self = Self(1000455012);
    pub const TESSELLATION_DOMAIN_ORIGIN_EXT: Self = Self(1000455002);
    pub const RASTERIZATION_STREAM_EXT: Self = Self(1000455013);
    pub const CONSERVATIVE_RASTERIZATION_MODE_EXT: Self = Self(1000455014);
    pub const EXTRA_PRIMITIVE_OVERESTIMATION_SIZE_EXT: Self = Self(1000455015);
    pub const DEPTH_CLIP_ENABLE_EXT: Self = Self(1000455016);
    pub const SAMPLE_LOCATIONS_ENABLE_EXT: Self = Self(1000455017);
    pub const COLOR_BLEND_ADVANCED_EXT: Self = Self(1000455018);
    pub const PROVOKING_VERTEX_MODE_EXT: Self = Self(1000455019);
    pub const LINE_RASTERIZATION_MODE_EXT: Self = Self(1000455020);
    pub const LINE_STIPPLE_ENABLE_EXT: Self = Self(1000455021);
    pub const DEPTH_CLIP_NEGATIVE_ONE_TO_ONE_EXT: Self = Self(1000455022);
    pub const VIEWPORT_W_SCALING_ENABLE_NV: Self = Self(1000455023);
    pub const VIEWPORT_SWIZZLE_NV: Self = Self(1000455024);
    pub const COVERAGE_TO_COLOR_ENABLE_NV: Self = Self(1000455025);
    pub const COVERAGE_TO_COLOR_LOCATION_NV: Self = Self(1000455026);
    pub const COVERAGE_MODULATION_MODE_NV: Self = Self(1000455027);
    pub const COVERAGE_MODULATION_TABLE_ENABLE_NV: Self = Self(1000455028);
    pub const COVERAGE_MODULATION_TABLE_NV: Self = Self(1000455029);
    pub const SHADING_RATE_IMAGE_ENABLE_NV: Self = Self(1000455030);
    pub const REPRESENTATIVE_FRAGMENT_TEST_ENABLE_NV: Self = Self(1000455031);
    pub const COVERAGE_REDUCTION_MODE_NV: Self = Self(1000455032);
    pub const ATTACHMENT_FEEDBACK_LOOP_ENABLE_EXT: Self = Self(1000524000);
    pub const DEPTH_CLAMP_RANGE_EXT: Self = Self(1000582000);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIFormat(i32);
impl RHIFormat {
    pub const UNDEFINED: Self = Self(0);
    pub const R4G4_UNORM_PACK8: Self = Self(1);
    pub const R4G4B4A4_UNORM_PACK16: Self = Self(2);
    pub const B4G4R4A4_UNORM_PACK16: Self = Self(3);
    pub const R5G6B5_UNORM_PACK16: Self = Self(4);
    pub const B5G6R5_UNORM_PACK16: Self = Self(5);
    pub const R5G5B5A1_UNORM_PACK16: Self = Self(6);
    pub const B5G5R5A1_UNORM_PACK16: Self = Self(7);
    pub const A1R5G5B5_UNORM_PACK16: Self = Self(8);
    pub const R8_UNORM: Self = Self(9);
    pub const R8_SNORM: Self = Self(10);
    pub const R8_USCALED: Self = Self(11);
    pub const R8_SSCALED: Self = Self(12);
    pub const R8_UINT: Self = Self(13);
    pub const R8_SINT: Self = Self(14);
    pub const R8_SRGB: Self = Self(15);
    pub const R8G8_UNORM: Self = Self(16);
    pub const R8G8_SNORM: Self = Self(17);
    pub const R8G8_USCALED: Self = Self(18);
    pub const R8G8_SSCALED: Self = Self(19);
    pub const R8G8_UINT: Self = Self(20);
    pub const R8G8_SINT: Self = Self(21);
    pub const R8G8_SRGB: Self = Self(22);
    pub const R8G8B8_UNORM: Self = Self(23);
    pub const R8G8B8_SNORM: Self = Self(24);
    pub const R8G8B8_USCALED: Self = Self(25);
    pub const R8G8B8_SSCALED: Self = Self(26);
    pub const R8G8B8_UINT: Self = Self(27);
    pub const R8G8B8_SINT: Self = Self(28);
    pub const R8G8B8_SRGB: Self = Self(29);
    pub const B8G8R8_UNORM: Self = Self(30);
    pub const B8G8R8_SNORM: Self = Self(31);
    pub const B8G8R8_USCALED: Self = Self(32);
    pub const B8G8R8_SSCALED: Self = Self(33);
    pub const B8G8R8_UINT: Self = Self(34);
    pub const B8G8R8_SINT: Self = Self(35);
    pub const B8G8R8_SRGB: Self = Self(36);
    pub const R8G8B8A8_UNORM: Self = Self(37);
    pub const R8G8B8A8_SNORM: Self = Self(38);
    pub const R8G8B8A8_USCALED: Self = Self(39);
    pub const R8G8B8A8_SSCALED: Self = Self(40);
    pub const R8G8B8A8_UINT: Self = Self(41);
    pub const R8G8B8A8_SINT: Self = Self(42);
    pub const R8G8B8A8_SRGB: Self = Self(43);
    pub const B8G8R8A8_UNORM: Self = Self(44);
    pub const B8G8R8A8_SNORM: Self = Self(45);
    pub const B8G8R8A8_USCALED: Self = Self(46);
    pub const B8G8R8A8_SSCALED: Self = Self(47);
    pub const B8G8R8A8_UINT: Self = Self(48);
    pub const B8G8R8A8_SINT: Self = Self(49);
    pub const B8G8R8A8_SRGB: Self = Self(50);
    pub const A8B8G8R8_UNORM_PACK32: Self = Self(51);
    pub const A8B8G8R8_SNORM_PACK32: Self = Self(52);
    pub const A8B8G8R8_USCALED_PACK32: Self = Self(53);
    pub const A8B8G8R8_SSCALED_PACK32: Self = Self(54);
    pub const A8B8G8R8_UINT_PACK32: Self = Self(55);
    pub const A8B8G8R8_SINT_PACK32: Self = Self(56);
    pub const A8B8G8R8_SRGB_PACK32: Self = Self(57);
    pub const A2R10G10B10_UNORM_PACK32: Self = Self(58);
    pub const A2R10G10B10_SNORM_PACK32: Self = Self(59);
    pub const A2R10G10B10_USCALED_PACK32: Self = Self(60);
    pub const A2R10G10B10_SSCALED_PACK32: Self = Self(61);
    pub const A2R10G10B10_UINT_PACK32: Self = Self(62);
    pub const A2R10G10B10_SINT_PACK32: Self = Self(63);
    pub const A2B10G10R10_UNORM_PACK32: Self = Self(64);
    pub const A2B10G10R10_SNORM_PACK32: Self = Self(65);
    pub const A2B10G10R10_USCALED_PACK32: Self = Self(66);
    pub const A2B10G10R10_SSCALED_PACK32: Self = Self(67);
    pub const A2B10G10R10_UINT_PACK32: Self = Self(68);
    pub const A2B10G10R10_SINT_PACK32: Self = Self(69);
    pub const R16_UNORM: Self = Self(70);
    pub const R16_SNORM: Self = Self(71);
    pub const R16_USCALED: Self = Self(72);
    pub const R16_SSCALED: Self = Self(73);
    pub const R16_UINT: Self = Self(74);
    pub const R16_SINT: Self = Self(75);
    pub const R16_SFLOAT: Self = Self(76);
    pub const R16G16_UNORM: Self = Self(77);
    pub const R16G16_SNORM: Self = Self(78);
    pub const R16G16_USCALED: Self = Self(79);
    pub const R16G16_SSCALED: Self = Self(80);
    pub const R16G16_UINT: Self = Self(81);
    pub const R16G16_SINT: Self = Self(82);
    pub const R16G16_SFLOAT: Self = Self(83);
    pub const R16G16B16_UNORM: Self = Self(84);
    pub const R16G16B16_SNORM: Self = Self(85);
    pub const R16G16B16_USCALED: Self = Self(86);
    pub const R16G16B16_SSCALED: Self = Self(87);
    pub const R16G16B16_UINT: Self = Self(88);
    pub const R16G16B16_SINT: Self = Self(89);
    pub const R16G16B16_SFLOAT: Self = Self(90);
    pub const R16G16B16A16_UNORM: Self = Self(91);
    pub const R16G16B16A16_SNORM: Self = Self(92);
    pub const R16G16B16A16_USCALED: Self = Self(93);
    pub const R16G16B16A16_SSCALED: Self = Self(94);
    pub const R16G16B16A16_UINT: Self = Self(95);
    pub const R16G16B16A16_SINT: Self = Self(96);
    pub const R16G16B16A16_SFLOAT: Self = Self(97);
    pub const R32_UINT: Self = Self(98);
    pub const R32_SINT: Self = Self(99);
    pub const R32_SFLOAT: Self = Self(100);
    pub const R32G32_UINT: Self = Self(101);
    pub const R32G32_SINT: Self = Self(102);
    pub const R32G32_SFLOAT: Self = Self(103);
    pub const R32G32B32_UINT: Self = Self(104);
    pub const R32G32B32_SINT: Self = Self(105);
    pub const R32G32B32_SFLOAT: Self = Self(106);
    pub const R32G32B32A32_UINT: Self = Self(107);
    pub const R32G32B32A32_SINT: Self = Self(108);
    pub const R32G32B32A32_SFLOAT: Self = Self(109);
    pub const R64_UINT: Self = Self(110);
    pub const R64_SINT: Self = Self(111);
    pub const R64_SFLOAT: Self = Self(112);
    pub const R64G64_UINT: Self = Self(113);
    pub const R64G64_SINT: Self = Self(114);
    pub const R64G64_SFLOAT: Self = Self(115);
    pub const R64G64B64_UINT: Self = Self(116);
    pub const R64G64B64_SINT: Self = Self(117);
    pub const R64G64B64_SFLOAT: Self = Self(118);
    pub const R64G64B64A64_UINT: Self = Self(119);
    pub const R64G64B64A64_SINT: Self = Self(120);
    pub const R64G64B64A64_SFLOAT: Self = Self(121);
    pub const B10G11R11_UFLOAT_PACK32: Self = Self(122);
    pub const E5B9G9R9_UFLOAT_PACK32: Self = Self(123);
    pub const D16_UNORM: Self = Self(124);
    pub const X8_D24_UNORM_PACK32: Self = Self(125);
    pub const D32_SFLOAT: Self = Self(126);
    pub const S8_UINT: Self = Self(127);
    pub const D16_UNORM_S8_UINT: Self = Self(128);
    pub const D24_UNORM_S8_UINT: Self = Self(129);
    pub const D32_SFLOAT_S8_UINT: Self = Self(130);
    pub const BC1_RGB_UNORM_BLOCK: Self = Self(131);
    pub const BC1_RGB_SRGB_BLOCK: Self = Self(132);
    pub const BC1_RGBA_UNORM_BLOCK: Self = Self(133);
    pub const BC1_RGBA_SRGB_BLOCK: Self = Self(134);
    pub const BC2_UNORM_BLOCK: Self = Self(135);
    pub const BC2_SRGB_BLOCK: Self = Self(136);
    pub const BC3_UNORM_BLOCK: Self = Self(137);
    pub const BC3_SRGB_BLOCK: Self = Self(138);
    pub const BC4_UNORM_BLOCK: Self = Self(139);
    pub const BC4_SNORM_BLOCK: Self = Self(140);
    pub const BC5_UNORM_BLOCK: Self = Self(141);
    pub const BC5_SNORM_BLOCK: Self = Self(142);
    pub const BC6H_UFLOAT_BLOCK: Self = Self(143);
    pub const BC6H_SFLOAT_BLOCK: Self = Self(144);
    pub const BC7_UNORM_BLOCK: Self = Self(145);
    pub const BC7_SRGB_BLOCK: Self = Self(146);
    pub const ETC2_R8G8B8_UNORM_BLOCK: Self = Self(147);
    pub const ETC2_R8G8B8_SRGB_BLOCK: Self = Self(148);
    pub const ETC2_R8G8B8A1_UNORM_BLOCK: Self = Self(149);
    pub const ETC2_R8G8B8A1_SRGB_BLOCK: Self = Self(150);
    pub const ETC2_R8G8B8A8_UNORM_BLOCK: Self = Self(151);
    pub const ETC2_R8G8B8A8_SRGB_BLOCK: Self = Self(152);
    pub const EAC_R11_UNORM_BLOCK: Self = Self(153);
    pub const EAC_R11_SNORM_BLOCK: Self = Self(154);
    pub const EAC_R11G11_UNORM_BLOCK: Self = Self(155);
    pub const EAC_R11G11_SNORM_BLOCK: Self = Self(156);
    pub const ASTC_4X4_UNORM_BLOCK: Self = Self(157);
    pub const ASTC_4X4_SRGB_BLOCK: Self = Self(158);
    pub const ASTC_5X4_UNORM_BLOCK: Self = Self(159);
    pub const ASTC_5X4_SRGB_BLOCK: Self = Self(160);
    pub const ASTC_5X5_UNORM_BLOCK: Self = Self(161);
    pub const ASTC_5X5_SRGB_BLOCK: Self = Self(162);
    pub const ASTC_6X5_UNORM_BLOCK: Self = Self(163);
    pub const ASTC_6X5_SRGB_BLOCK: Self = Self(164);
    pub const ASTC_6X6_UNORM_BLOCK: Self = Self(165);
    pub const ASTC_6X6_SRGB_BLOCK: Self = Self(166);
    pub const ASTC_8X5_UNORM_BLOCK: Self = Self(167);
    pub const ASTC_8X5_SRGB_BLOCK: Self = Self(168);
    pub const ASTC_8X6_UNORM_BLOCK: Self = Self(169);
    pub const ASTC_8X6_SRGB_BLOCK: Self = Self(170);
    pub const ASTC_8X8_UNORM_BLOCK: Self = Self(171);
    pub const ASTC_8X8_SRGB_BLOCK: Self = Self(172);
    pub const ASTC_10X5_UNORM_BLOCK: Self = Self(173);
    pub const ASTC_10X5_SRGB_BLOCK: Self = Self(174);
    pub const ASTC_10X6_UNORM_BLOCK: Self = Self(175);
    pub const ASTC_10X6_SRGB_BLOCK: Self = Self(176);
    pub const ASTC_10X8_UNORM_BLOCK: Self = Self(177);
    pub const ASTC_10X8_SRGB_BLOCK: Self = Self(178);
    pub const ASTC_10X10_UNORM_BLOCK: Self = Self(179);
    pub const ASTC_10X10_SRGB_BLOCK: Self = Self(180);
    pub const ASTC_12X10_UNORM_BLOCK: Self = Self(181);
    pub const ASTC_12X10_SRGB_BLOCK: Self = Self(182);
    pub const ASTC_12X12_UNORM_BLOCK: Self = Self(183);
    pub const ASTC_12X12_SRGB_BLOCK: Self = Self(184);
    pub const G8B8G8R8_422_UNORM: Self = Self(1000156000);
    pub const B8G8R8G8_422_UNORM: Self = Self(1000156001);
    pub const G8_B8_R8_3PLANE_420_UNORM: Self = Self(1000156002);
    pub const G8_B8R8_2PLANE_420_UNORM: Self = Self(1000156003);
    pub const G8_B8_R8_3PLANE_422_UNORM: Self = Self(1000156004);
    pub const G8_B8R8_2PLANE_422_UNORM: Self = Self(1000156005);
    pub const G8_B8_R8_3PLANE_444_UNORM: Self = Self(1000156006);
    pub const R10X6_UNORM_PACK16: Self = Self(1000156007);
    pub const R10X6G10X6_UNORM_2PACK16: Self = Self(1000156008);
    pub const R10X6G10X6B10X6A10X6_UNORM_4PACK16: Self = Self(1000156009);
    pub const G10X6B10X6G10X6R10X6_422_UNORM_4PACK16: Self = Self(1000156010);
    pub const B10X6G10X6R10X6G10X6_422_UNORM_4PACK16: Self = Self(1000156011);
    pub const G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16: Self = Self(1000156012);
    pub const G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16: Self = Self(1000156013);
    pub const G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16: Self = Self(1000156014);
    pub const G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16: Self = Self(1000156015);
    pub const G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16: Self = Self(1000156016);
    pub const R12X4_UNORM_PACK16: Self = Self(1000156017);
    pub const R12X4G12X4_UNORM_2PACK16: Self = Self(1000156018);
    pub const R12X4G12X4B12X4A12X4_UNORM_4PACK16: Self = Self(1000156019);
    pub const G12X4B12X4G12X4R12X4_422_UNORM_4PACK16: Self = Self(1000156020);
    pub const B12X4G12X4R12X4G12X4_422_UNORM_4PACK16: Self = Self(1000156021);
    pub const G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16: Self = Self(1000156022);
    pub const G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16: Self = Self(1000156023);
    pub const G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16: Self = Self(1000156024);
    pub const G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16: Self = Self(1000156025);
    pub const G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16: Self = Self(1000156026);
    pub const G16B16G16R16_422_UNORM: Self = Self(1000156027);
    pub const B16G16R16G16_422_UNORM: Self = Self(1000156028);
    pub const G16_B16_R16_3PLANE_420_UNORM: Self = Self(1000156029);
    pub const G16_B16R16_2PLANE_420_UNORM: Self = Self(1000156030);
    pub const G16_B16_R16_3PLANE_422_UNORM: Self = Self(1000156031);
    pub const G16_B16R16_2PLANE_422_UNORM: Self = Self(1000156032);
    pub const G16_B16_R16_3PLANE_444_UNORM: Self = Self(1000156033);
    pub const G8_B8R8_2PLANE_444_UNORM: Self = Self(1000330000);
    pub const G10X6_B10X6R10X6_2PLANE_444_UNORM_3PACK16: Self = Self(1000330001);
    pub const G12X4_B12X4R12X4_2PLANE_444_UNORM_3PACK16: Self = Self(1000330002);
    pub const G16_B16R16_2PLANE_444_UNORM: Self = Self(1000330003);
    pub const A4R4G4B4_UNORM_PACK16: Self = Self(1000340000);
    pub const A4B4G4R4_UNORM_PACK16: Self = Self(1000340001);
    pub const ASTC_4X4_SFLOAT_BLOCK: Self = Self(1000066000);
    pub const ASTC_5X4_SFLOAT_BLOCK: Self = Self(1000066001);
    pub const ASTC_5X5_SFLOAT_BLOCK: Self = Self(1000066002);
    pub const ASTC_6X5_SFLOAT_BLOCK: Self = Self(1000066003);
    pub const ASTC_6X6_SFLOAT_BLOCK: Self = Self(1000066004);
    pub const ASTC_8X5_SFLOAT_BLOCK: Self = Self(1000066005);
    pub const ASTC_8X6_SFLOAT_BLOCK: Self = Self(1000066006);
    pub const ASTC_8X8_SFLOAT_BLOCK: Self = Self(1000066007);
    pub const ASTC_10X5_SFLOAT_BLOCK: Self = Self(1000066008);
    pub const ASTC_10X6_SFLOAT_BLOCK: Self = Self(1000066009);
    pub const ASTC_10X8_SFLOAT_BLOCK: Self = Self(1000066010);
    pub const ASTC_10X10_SFLOAT_BLOCK: Self = Self(1000066011);
    pub const ASTC_12X10_SFLOAT_BLOCK: Self = Self(1000066012);
    pub const ASTC_12X12_SFLOAT_BLOCK: Self = Self(1000066013);
    pub const A1B5G5R5_UNORM_PACK16: Self = Self(1000470000);
    pub const A8_UNORM: Self = Self(1000470001);
    pub const PVRTC1_2BPP_UNORM_BLOCK_IMG: Self = Self(1000054000);
    pub const PVRTC1_4BPP_UNORM_BLOCK_IMG: Self = Self(1000054001);
    pub const PVRTC2_2BPP_UNORM_BLOCK_IMG: Self = Self(1000054002);
    pub const PVRTC2_4BPP_UNORM_BLOCK_IMG: Self = Self(1000054003);
    pub const PVRTC1_2BPP_SRGB_BLOCK_IMG: Self = Self(1000054004);
    pub const PVRTC1_4BPP_SRGB_BLOCK_IMG: Self = Self(1000054005);
    pub const PVRTC2_2BPP_SRGB_BLOCK_IMG: Self = Self(1000054006);
    pub const PVRTC2_4BPP_SRGB_BLOCK_IMG: Self = Self(1000054007);
    pub const R8_BOOL_ARM: Self = Self(1000460000);
    pub const R16G16_SFIXED5_NV: Self = Self(1000464000);
    pub const R10X6_UINT_PACK16_ARM: Self = Self(1000609000);
    pub const R10X6G10X6_UINT_2PACK16_ARM: Self = Self(1000609001);
    pub const R10X6G10X6B10X6A10X6_UINT_4PACK16_ARM: Self = Self(1000609002);
    pub const R12X4_UINT_PACK16_ARM: Self = Self(1000609003);
    pub const R12X4G12X4_UINT_2PACK16_ARM: Self = Self(1000609004);
    pub const R12X4G12X4B12X4A12X4_UINT_4PACK16_ARM: Self = Self(1000609005);
    pub const R14X2_UINT_PACK16_ARM: Self = Self(1000609006);
    pub const R14X2G14X2_UINT_2PACK16_ARM: Self = Self(1000609007);
    pub const R14X2G14X2B14X2A14X2_UINT_4PACK16_ARM: Self = Self(1000609008);
    pub const R14X2_UNORM_PACK16_ARM: Self = Self(1000609009);
    pub const R14X2G14X2_UNORM_2PACK16_ARM: Self = Self(1000609010);
    pub const R14X2G14X2B14X2A14X2_UNORM_4PACK16_ARM: Self = Self(1000609011);
    pub const G14X2_B14X2R14X2_2PLANE_420_UNORM_3PACK16_ARM: Self = Self(1000609012);
    pub const G14X2_B14X2R14X2_2PLANE_422_UNORM_3PACK16_ARM: Self = Self(1000609013);

    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIFrontFace(i32);
impl RHIFrontFace {
    pub const COUNTER_CLOCKWISE: Self = Self(0);
    pub const CLOCKWISE: Self = Self(1);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIImageLayout(i32);
impl RHIImageLayout {
    pub const UNDEFINED: Self = Self(0);
    pub const GENERAL: Self = Self(1);
    pub const COLOR_ATTACHMENT_OPTIMAL: Self = Self(2);
    pub const DEPTH_STENCIL_ATTACHMENT_OPTIMAL: Self = Self(3);
    pub const DEPTH_STENCIL_READ_ONLY_OPTIMAL: Self = Self(4);
    pub const SHADER_READ_ONLY_OPTIMAL: Self = Self(5);
    pub const TRANSFER_SRC_OPTIMAL: Self = Self(6);
    pub const TRANSFER_DST_OPTIMAL: Self = Self(7);
    pub const PREINITIALIZED: Self = Self(8);
    pub const DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL: Self = Self(1000117000);
    pub const DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL: Self = Self(1000117001);
    pub const DEPTH_ATTACHMENT_OPTIMAL: Self = Self(1000241000);
    pub const DEPTH_READ_ONLY_OPTIMAL: Self = Self(1000241001);
    pub const STENCIL_ATTACHMENT_OPTIMAL: Self = Self(1000241002);
    pub const STENCIL_READ_ONLY_OPTIMAL: Self = Self(1000241003);
    pub const READ_ONLY_OPTIMAL: Self = Self(1000314000);
    pub const ATTACHMENT_OPTIMAL: Self = Self(1000314001);
    pub const RENDERING_LOCAL_READ: Self = Self(1000232000);
    pub const PRESENT_SRC_KHR: Self = Self(1000001002);
    pub const VIDEO_DECODE_DST_KHR: Self = Self(1000024000);
    pub const VIDEO_DECODE_SRC_KHR: Self = Self(1000024001);
    pub const VIDEO_DECODE_DPB_KHR: Self = Self(1000024002);
    pub const SHARED_PRESENT_KHR: Self = Self(1000111000);
    pub const FRAGMENT_DENSITY_MAP_OPTIMAL_EXT: Self = Self(1000218000);
    pub const FRAGMENT_SHADING_RATE_ATTACHMENT_OPTIMAL_KHR: Self = Self(1000164003);
    pub const VIDEO_ENCODE_DST_KHR: Self = Self(1000299000);
    pub const VIDEO_ENCODE_SRC_KHR: Self = Self(1000299001);
    pub const VIDEO_ENCODE_DPB_KHR: Self = Self(1000299002);
    pub const ATTACHMENT_FEEDBACK_LOOP_OPTIMAL_EXT: Self = Self(1000339000);
    pub const TENSOR_ALIASING_ARM: Self = Self(1000460000);
    pub const VIDEO_ENCODE_QUANTIZATION_MAP_KHR: Self = Self(1000553000);
    pub const ZERO_INITIALIZED_EXT: Self = Self(1000620000);

    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHILogicOp(i32);
impl RHILogicOp {
    pub const CLEAR: Self = Self(0);
    pub const AND: Self = Self(1);
    pub const AND_REVERSE: Self = Self(2);
    pub const COPY: Self = Self(3);
    pub const AND_INVERTED: Self = Self(4);
    pub const NO_OP: Self = Self(5);
    pub const XOR: Self = Self(6);
    pub const OR: Self = Self(7);
    pub const NOR: Self = Self(8);
    pub const EQUIVALENT: Self = Self(9);
    pub const INVERT: Self = Self(10);
    pub const OR_REVERSE: Self = Self(11);
    pub const COPY_INVERTED: Self = Self(12);
    pub const OR_INVERTED: Self = Self(13);
    pub const NAND: Self = Self(14);
    pub const SET: Self = Self(15);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIPipelineBindPoint(i32);
impl RHIPipelineBindPoint {
    pub const GRAPHICS: Self = Self(0);
    pub const COMPUTE: Self = Self(1);
    pub const EXECUTION_GRAPH_AMDX: Self = Self(1000134000);
    pub const RAY_TRACING_KHR: Self = Self(1000165000);
    pub const SUBPASS_SHADING_HUAWEI: Self = Self(1000369003);
    pub const DATA_GRAPH_ARM: Self = Self(1000507000);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIPolygonMode(i32);
impl RHIPolygonMode {
    pub const FILL: Self = Self(0);
    pub const LINE: Self = Self(1);
    pub const POINT: Self = Self(2);
    pub const FILL_RECTANGLE_NV: Self = Self(1000153000);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIPrimitiveTopology(i32);
impl RHIPrimitiveTopology {
    pub const POINT_LIST: Self = Self(0);
    pub const LINE_LIST: Self = Self(1);
    pub const LINE_STRIP: Self = Self(2);
    pub const TRIANGLE_LIST: Self = Self(3);
    pub const TRIANGLE_STRIP: Self = Self(4);
    pub const TRIANGLE_FAN: Self = Self(5);
    pub const LINE_LIST_WITH_ADJACENCY: Self = Self(6);
    pub const LINE_STRIP_WITH_ADJACENCY: Self = Self(7);
    pub const TRIANGLE_LIST_WITH_ADJACENCY: Self = Self(8);
    pub const TRIANGLE_STRIP_WITH_ADJACENCY: Self = Self(9);
    pub const PATCH_LIST: Self = Self(10);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHISubpassContents(i32);
impl RHISubpassContents {
    pub const INLINE: Self = Self(0);
    pub const SECONDARY_COMMAND_BUFFERS: Self = Self(1);
    pub const INLINE_AND_SECONDARY_COMMAND_BUFFERS_KHR: Self = Self(1000451000);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIVertexInputRate(i32);
impl RHIVertexInputRate {
    pub const VERTEX: Self = Self(0);
    pub const INSTANCE: Self = Self(1);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RHIStencilOp(i32);
impl RHIStencilOp {
    pub const KEEP: Self = Self(0);
    pub const ZERO: Self = Self(1);
    pub const REPLACE: Self = Self(2);
    pub const INCREMENT_AND_CLAMP: Self = Self(3);
    pub const DECREMENT_AND_CLAMP: Self = Self(4);
    pub const INVERT: Self = Self(5);
    pub const INCREMENT_AND_WRAP: Self = Self(6);
    pub const DECREMENT_AND_WRAP: Self = Self(7);

    /// Constructs an instance of this enum with the supplied underlying value.
    #[inline]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Gets the underlying value for this enum instance.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIDescriptorSetLayoutCreateFlags: Flags {
        const PUSH_DESCRIPTOR = 1;
        const UPDATE_AFTER_BIND_POOL = 1 << 1;
        const HOST_ONLY_POOL_EXT = 1 << 2;
        const DESCRIPTOR_BUFFER_EXT = 1 << 4;
        const EMBEDDED_IMMUTABLE_SAMPLERS_EXT = 1 << 5;
        const PER_STAGE_NV = 1 << 6;
        const INDIRECT_BINDABLE_NV = 1 << 7;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIShaderStageFlags: Flags {
        const VERTEX = 1;
        const TESSELLATION_CONTROL = 1 << 1;
        const TESSELLATION_EVALUATION = 1 << 2;
        const GEOMETRY = 1 << 3;
        const FRAGMENT = 1 << 4;
        const ALL_GRAPHICS = Self::VERTEX.bits() | Self::TESSELLATION_CONTROL.bits() | Self::TESSELLATION_EVALUATION.bits() | Self::GEOMETRY.bits() | Self::FRAGMENT.bits();
        const COMPUTE = 1 << 5;
        const TASK_EXT = 1 << 6;
        const MESH_EXT = 1 << 7;
        const RAYGEN_KHR = 1 << 8;
        const ANY_HIT_KHR = 1 << 9;
        const CLOSEST_HIT_KHR = 1 << 10;
        const MISS_KHR = 1 << 11;
        const INTERSECTION_KHR = 1 << 12;
        const CALLABLE_KHR = 1 << 13;
        const SUBPASS_SHADING_HUAWEI = 1 << 14;
        const CLUSTER_CULLING_HUAWEI = 1 << 19;
        const ALL = i32::MAX as u32;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIAccessFlags: Flags {
        const INDIRECT_COMMAND_READ = 1;
        const INDEX_READ = 1 << 1;
        const VERTEX_ATTRIBUTE_READ = 1 << 2;
        const UNIFORM_READ = 1 << 3;
        const INPUT_ATTACHMENT_READ = 1 << 4;
        const SHADER_READ = 1 << 5;
        const SHADER_WRITE = 1 << 6;
        const COLOR_ATTACHMENT_READ = 1 << 7;
        const COLOR_ATTACHMENT_WRITE = 1 << 8;
        const DEPTH_STENCIL_ATTACHMENT_READ = 1 << 9;
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 1 << 10;
        const TRANSFER_READ = 1 << 11;
        const TRANSFER_WRITE = 1 << 12;
        const HOST_READ = 1 << 13;
        const HOST_WRITE = 1 << 14;
        const MEMORY_READ = 1 << 15;
        const MEMORY_WRITE = 1 << 16;
        const COMMAND_PREPROCESS_READ_EXT = 1 << 17;
        const COMMAND_PREPROCESS_WRITE_EXT = 1 << 18;
        const COLOR_ATTACHMENT_READ_NONCOHERENT_EXT = 1 << 19;
        const CONDITIONAL_RENDERING_READ_EXT = 1 << 20;
        const ACCELERATION_STRUCTURE_READ_KHR = 1 << 21;
        const ACCELERATION_STRUCTURE_WRITE_KHR = 1 << 22;
        const FRAGMENT_SHADING_RATE_ATTACHMENT_READ_KHR = 1 << 23;
        const FRAGMENT_DENSITY_MAP_READ_EXT = 1 << 24;
        const TRANSFORM_FEEDBACK_WRITE_EXT = 1 << 25;
        const TRANSFORM_FEEDBACK_COUNTER_READ_EXT = 1 << 26;
        const TRANSFORM_FEEDBACK_COUNTER_WRITE_EXT = 1 << 27;
    }
}
pub type RHIImageAspectFlags = u32;
pub type RHIFormatFeatureFlags = u32;
pub type RHIImageCreateFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHISampleCountFlags: Flags {
        const _1 = 1;
        const _2 = 1 << 1;
        const _4 = 1 << 2;
        const _8 = 1 << 3;
        const _16 = 1 << 4;
        const _32 = 1 << 5;
        const _64 = 1 << 6;
    }
}
pub type RHIImageUsageFlags = u32;
pub type RHIInstanceCreateFlags = u32;
pub type RHIMemoryHeapFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIMemoryPropertyFlags: Flags {
        const DEVICE_LOCAL = 1;
        const HOST_VISIBLE = 1 << 1;
        const HOST_COHERENT = 1 << 2;
        const HOST_CACHED = 1 << 3;
        const LAZILY_ALLOCATED = 1 << 4;
        const PROTECTED = 1 << 5;
        const DEVICE_COHERENT_AMD = 1 << 6;
        const DEVICE_UNCACHED_AMD = 1 << 7;
        const RDMA_CAPABLE_NV = 1 << 8;
    }
}
pub type RHIQueueFlags = u32;
pub type RHIDeviceCreateFlags = u32;
pub type RHIDeviceQueueCreateFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineStageFlags: Flags {
        const TOP_OF_PIPE = 1;
        const DRAW_INDIRECT = 1 << 1;
        const VERTEX_INPUT = 1 << 2;
        const VERTEX_SHADER = 1 << 3;
        const TESSELLATION_CONTROL_SHADER = 1 << 4;
        const TESSELLATION_EVALUATION_SHADER = 1 << 5;
        const GEOMETRY_SHADER = 1 << 6;
        const FRAGMENT_SHADER = 1 << 7;
        const EARLY_FRAGMENT_TESTS = 1 << 8;
        const LATE_FRAGMENT_TESTS = 1 << 9;
        const COLOR_ATTACHMENT_OUTPUT = 1 << 10;
        const COMPUTE_SHADER = 1 << 11;
        const TRANSFER = 1 << 12;
        const BOTTOM_OF_PIPE = 1 << 13;
        const HOST = 1 << 14;
        const ALL_GRAPHICS = 1 << 15;
        const ALL_COMMANDS = 1 << 16;
        const COMMAND_PREPROCESS_EXT = 1 << 17;
        const CONDITIONAL_RENDERING_EXT = 1 << 18;
        const TASK_SHADER_EXT = 1 << 19;
        const MESH_SHADER_EXT = 1 << 20;
        const RAY_TRACING_SHADER_KHR = 1 << 21;
        const FRAGMENT_SHADING_RATE_ATTACHMENT_KHR = 1 << 22;
        const FRAGMENT_DENSITY_PROCESS_EXT = 1 << 23;
        const TRANSFORM_FEEDBACK_EXT = 1 << 24;
        const ACCELERATION_STRUCTURE_BUILD_KHR = 1 << 25;
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIMemoryMapFlags: Flags {
        const PLACED_EXT = 1;
    }
}
pub type RHISparseMemoryBindFlags = u32;
pub type RHISparseImageFormatFlags = u32;
pub type RHIFenceCreateFlags = u32;
pub type RHISemaphoreCreateFlags = u32;
pub type RHIEventCreateFlags = u32;
pub type RHIQueryPipelineStatisticFlags = u32;
pub type RHIQueryPoolCreateFlags = u32;
pub type RHIQueryResultFlags = u32;
pub type RHIBufferCreateFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIBufferUsageFlags: Flags {
        const TRANSFER_SRC = 1;
        const TRANSFER_DST = 1 << 1;
        const UNIFORM_TEXEL_BUFFER = 1 << 2;
        const STORAGE_TEXEL_BUFFER = 1 << 3;
        const UNIFORM_BUFFER = 1 << 4;
        const STORAGE_BUFFER = 1 << 5;
        const INDEX_BUFFER = 1 << 6;
        const VERTEX_BUFFER = 1 << 7;
        const INDIRECT_BUFFER = 1 << 8;
        const CONDITIONAL_RENDERING_EXT = 1 << 9;
        const SHADER_BINDING_TABLE_KHR = 1 << 10;
        const TRANSFORM_FEEDBACK_BUFFER_EXT = 1 << 11;
        const TRANSFORM_FEEDBACK_COUNTER_BUFFER_EXT = 1 << 12;
        const VIDEO_DECODE_SRC_KHR = 1 << 13;
        const VIDEO_DECODE_DST_KHR = 1 << 14;
        const VIDEO_ENCODE_DST_KHR = 1 << 15;
        const VIDEO_ENCODE_SRC_KHR = 1 << 16;
        const SHADER_DEVICE_ADDRESS = 1 << 17;
        const ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR = 1 << 19;
        const ACCELERATION_STRUCTURE_STORAGE_KHR = 1 << 20;
        const SAMPLER_DESCRIPTOR_BUFFER_EXT = 1 << 21;
        const RESOURCE_DESCRIPTOR_BUFFER_EXT = 1 << 22;
        const MICROMAP_BUILD_INPUT_READ_ONLY_EXT = 1 << 23;
        const MICROMAP_STORAGE_EXT = 1 << 24;
        const EXECUTION_GRAPH_SCRATCH_AMDX = 1 << 25;
        const PUSH_DESCRIPTORS_DESCRIPTOR_BUFFER_EXT = 1 << 26;
        const TILE_MEMORY_QCOM = 1 << 27;
    }
}
pub type RHIBufferViewCreateFlags = u32;
pub type RHIImageViewCreateFlags = u32;
pub type RHIShaderModuleCreateFlags = u32;
pub type RHIPipelineCacheCreateFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIColorComponentFlags: Flags {
        const R = 1;
        const G = 1 << 1;
        const B = 1 << 2;
        const A = 1 << 3;
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineCreateFlags: Flags {
        const DISABLE_OPTIMIZATION = 1;
        const ALLOW_DERIVATIVES = 1 << 1;
        const DERIVATIVE = 1 << 2;
        const VIEW_INDEX_FROM_DEVICE_INDEX = 1 << 3;
        const DISPATCH_BASE = 1 << 4;
        const DEFER_COMPILE_NV = 1 << 5;
        const CAPTURE_STATISTICS_KHR = 1 << 6;
        const CAPTURE_INTERNAL_REPRESENTATIONS_KHR = 1 << 7;
        const FAIL_ON_PIPELINE_COMPILE_REQUIRED = 1 << 8;
        const EARLY_RETURN_ON_FAILURE = 1 << 9;
        const LINK_TIME_OPTIMIZATION_EXT = 1 << 10;
        const LIBRARY_KHR = 1 << 11;
        const RAY_TRACING_SKIP_TRIANGLES_KHR = 1 << 12;
        const RAY_TRACING_SKIP_AABBS_KHR = 1 << 13;
        const RAY_TRACING_NO_NULL_ANY_HIT_SHADERS_KHR = 1 << 14;
        const RAY_TRACING_NO_NULL_CLOSEST_HIT_SHADERS_KHR = 1 << 15;
        const RAY_TRACING_NO_NULL_MISS_SHADERS_KHR = 1 << 16;
        const RAY_TRACING_NO_NULL_INTERSECTION_SHADERS_KHR = 1 << 17;
        const INDIRECT_BINDABLE_NV = 1 << 18;
        const RAY_TRACING_SHADER_GROUP_HANDLE_CAPTURE_REPLAY_KHR = 1 << 19;
        const RAY_TRACING_ALLOW_MOTION_NV = 1 << 20;
        const RENDERING_FRAGMENT_SHADING_RATE_ATTACHMENT_KHR = 1 << 21;
        const RENDERING_FRAGMENT_DENSITY_MAP_ATTACHMENT_EXT = 1 << 22;
        const RETAIN_LINK_TIME_OPTIMIZATION_INFO_EXT = 1 << 23;
        const RAY_TRACING_OPACITY_MICROMAP_EXT = 1 << 24;
        const COLOR_ATTACHMENT_FEEDBACK_LOOP_EXT = 1 << 25;
        const DEPTH_STENCIL_ATTACHMENT_FEEDBACK_LOOP_EXT = 1 << 26;
        const NO_PROTECTED_ACCESS = 1 << 27;
        const RAY_TRACING_DISPLACEMENT_MICROMAP_NV = 1 << 28;
        const DESCRIPTOR_BUFFER_EXT = 1 << 29;
        const PROTECTED_ACCESS_ONLY = 1 << 30;
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineShaderStageCreateFlags: Flags {
        const ALLOW_VARYING_SUBGROUP_SIZE = 1;
        const REQUIRE_FULL_SUBGROUPS = 1 << 1;
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHICullModeFlags: Flags {
        const NONE = 0;
        const FRONT = 1;
        const BACK = 1 << 1;
        const FRONT_AND_BACK = Self::FRONT.bits() | Self::BACK.bits();
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineVertexInputStateCreateFlags: Flags { }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineInputAssemblyStateCreateFlags: Flags { }
}
pub type RHIPipelineTessellationStateCreateFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineViewportStateCreateFlags: Flags { }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineRasterizationStateCreateFlags: Flags { }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineMultisampleStateCreateFlags: Flags { }
}
pub type RHIPipelineDepthStencilStateCreateFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineColorBlendStateCreateFlags: Flags {
        const RASTERIZATION_ORDER_ATTACHMENT_ACCESS_EXT = 1;
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineDynamicStateCreateFlags: Flags { }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIPipelineLayoutCreateFlags: Flags {
        const INDEPENDENT_SETS_EXT = 1 << 1;
    }
}
pub type RHISamplerCreateFlags = u32;
pub type RHIDescriptorPoolCreateFlags = u32;
pub type RHIDescriptorPoolResetFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIAttachmentDescriptionFlags: Flags {
        const MAY_ALIAS = 1;
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIDependencyFlags: Flags {
        const BY_REGION = 1;
        const VIEW_LOCAL = 1 << 1;
        const DEVICE_GROUP = 1 << 2;
        const FEEDBACK_LOOP_EXT = 1 << 3;
        const QUEUE_FAMILY_OWNERSHIP_TRANSFER_USE_ALL_STAGES_KHR = 1 << 5;
        const ASYMMETRIC_EVENT_KHR = 1 << 6;
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIFramebufferCreateFlags: Flags {
        const IMAGELESS = 1;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHIRenderPassCreateFlags: Flags {
        const TRANSFORM_QCOM = 1 << 1;
        const PER_LAYER_FRAGMENT_DENSITY_VALVE = 1 << 2;
    }
}
pub type RHICommandPoolCreateFlags = u32;
bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct RHISubpassDescriptionFlags: Flags {
        const PER_VIEW_ATTRIBUTES_NVX = 1;
        const PER_VIEW_POSITION_X_ONLY_NVX = 1 << 1;
        const FRAGMENT_REGION_QCOM = 1 << 2;
        const SHADER_RESOLVE_QCOM = 1 << 3;
        const RASTERIZATION_ORDER_ATTACHMENT_COLOR_ACCESS_EXT = 1 << 4;
        const RASTERIZATION_ORDER_ATTACHMENT_DEPTH_ACCESS_EXT = 1 << 5;
        const RASTERIZATION_ORDER_ATTACHMENT_STENCIL_ACCESS_EXT = 1 << 6;
        const ENABLE_LEGACY_DITHERING_EXT = 1 << 7;
        const TILE_SHADING_APRON_QCOM = 1 << 8;
    }
}
pub type RHICommandPoolResetFlags = u32;
pub type RHICommandBufferUsageFlags = u32;
pub type RHIQueryControlFlags = u32;
pub type RHICommandBufferResetFlags = u32;
pub type RHIStencilFaceFlags = u32;
pub type RHIBool32 = u32;
pub type RHIDeviceAddress = u64;
pub type RHIDeviceSize = u64;
pub type RHIFlags = u32;
pub type RHISampleMask = u32;

pub enum RenderPipelineType{
    ForwardPipeline,
    DeferredPipeline,
    PipelineTypeCount,
}

#[derive(Default)]
pub struct BufferData {
    pub m_data: Vec<u8>,
}

#[repr(C)]
pub struct MeshVertexDataDefinition {
    pub x: f32, pub y: f32, pub z: f32,
    pub nx: f32,pub ny: f32, pub nz: f32,
    pub tx: f32,pub ty: f32, pub tz: f32,
    pub u: f32,pub v: f32,
}

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct MeshSourceDesc{
    pub m_mesh_file: String,
}


#[derive(Default)]
pub struct StaticMeshData{
    pub m_vertex_buffer: BufferData,
    pub m_index_buffer: BufferData,
}

#[derive(Default)]
pub struct RenderMeshData{
    pub m_static_mesh_data: StaticMeshData,
    pub m_skeleton_binding_buffer: BufferData,
}