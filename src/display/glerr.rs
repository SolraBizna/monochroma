/// Maps every valid value returnable by `glGetError` to its name.
///
/// Useful for any OpenGL-based renderer.
pub const ERROR_TABLE: &[(u32, &str)] = &[
    (0x0500, "GL_INVALID_ENUM"),
    (0x0501, "GL_INVALID_VALUE"),
    (0x0502, "GL_INVALID_OPERATION"),
    (0x0503, "GL_STACK_OVERFLOW"),
    (0x0504, "GL_STACK_UNDERFLOW"),
    (0x0505, "GL_OUT_OF_MEMORY"),
    (0x0506, "GL_INVALID_FRAMEBUFFER_OPERATION"),
    (0x8031, "GL_TABLE_TOO_LARGE"),
    // at the time of this writing, this one is only present in an extension
    (0x8065, "GL_TEXTURE_TOO_LARGE"),
];
