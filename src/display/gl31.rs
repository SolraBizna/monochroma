#![allow(
    dead_code,
    non_snake_case,
    non_upper_case_globals,
    unused_imports,
    clippy::all
)]

//! This module was generated using the rglgen crate.
//! It is a partial binding for OpenGL 3.1.
//!
//! It includes support for the following extensions:
//! - GL_ARB_debug_output

// The following comments are from the source XML file. It refers to that file,
// not this generated Rust code. Nevertheless, valuable copyright and
// provenance data may be present.
//
// Copyright 2013-2020 The Khronos Group Inc.
// SPDX-License-Identifier: Apache-2.0
//
// This file, gl.xml, is the OpenGL and OpenGL API Registry. The canonical
// version of the registry, together with documentation, schema, and Python
// generator scripts used to generate C header files for OpenGL and OpenGL ES,
// can always be found in the Khronos Registry at
// https://github.com/KhronosGroup/OpenGL-Registry
//

// *** TYPES ***
pub type GLenum = libc::c_uint;
pub type GLboolean = libc::c_uchar;
pub type GLbitfield = libc::c_uint;
// Not an actual GL type, though used in headers in the past
pub type GLvoid = libc::c_void;
pub type GLubyte = u8;
pub type GLint = libc::c_int;
pub type GLuint = libc::c_uint;
pub type GLsizei = libc::c_int;
pub type GLfloat = f32;
pub type GLclampf = f32;
pub type GLclampd = libc::c_double;
pub type GLchar = libc::c_char;
pub type GLhalf = u16;
pub type GLsizeiptr = isize;
pub type GLDEBUGPROCARB = Option<
    extern "C" fn(
        source: GLenum,
        r#type: GLenum,
        id: GLuint,
        severity: GLenum,
        length: GLsizei,
        message: *const GLchar,
        userParam: *const libc::c_void,
    ),
>;

// *** VALUES ***
pub const GL_ARRAY_BUFFER: u32 = 0x8892;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
pub const GL_COMPILE_STATUS: u32 = 0x8b81;
pub const GL_DEBUG_OUTPUT_SYNCHRONOUS_ARB: u32 = 0x8242;
pub const GL_DEBUG_SEVERITY_HIGH_ARB: u32 = 0x9146;
pub const GL_DEBUG_SEVERITY_LOW_ARB: u32 = 0x9148;
pub const GL_DEBUG_SEVERITY_MEDIUM_ARB: u32 = 0x9147;
pub const GL_DEBUG_SOURCE_API_ARB: u32 = 0x8246;
pub const GL_DEBUG_SOURCE_APPLICATION_ARB: u32 = 0x824a;
pub const GL_DEBUG_SOURCE_OTHER_ARB: u32 = 0x824b;
pub const GL_DEBUG_SOURCE_SHADER_COMPILER_ARB: u32 = 0x8248;
pub const GL_DEBUG_SOURCE_THIRD_PARTY_ARB: u32 = 0x8249;
pub const GL_DEBUG_SOURCE_WINDOW_SYSTEM_ARB: u32 = 0x8247;
pub const GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR_ARB: u32 = 0x824d;
pub const GL_DEBUG_TYPE_ERROR_ARB: u32 = 0x824c;
pub const GL_DEBUG_TYPE_OTHER_ARB: u32 = 0x8251;
pub const GL_DEBUG_TYPE_PERFORMANCE_ARB: u32 = 0x8250;
pub const GL_DEBUG_TYPE_PORTABILITY_ARB: u32 = 0x824f;
pub const GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR_ARB: u32 = 0x824e;
pub const GL_EXTENSIONS: u32 = 0x1f03;
pub const GL_FLOAT: u32 = 0x1406;
pub const GL_FRAGMENT_SHADER: u32 = 0x8b30;
pub const GL_FRAMEBUFFER_SRGB: u32 = 0x8db9;
pub const GL_INFO_LOG_LENGTH: u32 = 0x8b84;
pub const GL_LINK_STATUS: u32 = 0x8b82;
pub const GL_NEAREST: u32 = 0x2600;
pub const GL_NUM_EXTENSIONS: u32 = 0x821d;
pub const GL_R32UI: u32 = 0x8236;
pub const GL_RED_INTEGER: u32 = 0x8d94;
pub const GL_STREAM_DRAW: u32 = 0x88e0;
pub const GL_TEXTURE_2D: u32 = 0xde1;
pub const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
pub const GL_TRIANGLE_STRIP: u32 = 0x5;
pub const GL_UNSIGNED_INT: u32 = 0x1405;
pub const GL_VERTEX_SHADER: u32 = 0x8b31;

// *** COMMANDS ***
pub struct Procs {
    procs: [*const (); 34],
    pub has_ARB_debug_output: bool,
}

use std::fmt;
impl fmt::Debug for Procs {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Procs{{...}}")?;
        Ok(())
    }
}
extern "C" fn glDebugMessageCallbackARB_null_imp(
    _: GLDEBUGPROCARB,
    _: *const libc::c_void,
) -> libc::c_void {
    missing_ext_panic("glDebugMessageCallbackARB", "GL_ARB_debug_output");
}

#[inline(never)]
fn missing_ext_panic(name: &str, ext: &str) -> ! {
    panic!(
        "{} called, but the requisite extension ({}) is not present",
        name, ext
    );
}

use std::ffi::CStr;
use std::mem::{transmute, MaybeUninit};
impl Procs {
    pub fn new<E, F: Fn(&[u8]) -> Result<*const (), E>>(
        get_proc: F,
    ) -> Result<Procs, E> {
        let mut procs: [MaybeUninit<*const ()>; 34] =
            unsafe { MaybeUninit::uninit().assume_init() };
        Procs::getprocs(
            &get_proc,
            &mut procs[0..33],
            &[
                b"glAttachShader\0",
                b"glBindBuffer\0",
                b"glBindTexture\0",
                b"glBindVertexArray\0",
                b"glBufferData\0",
                b"glClear\0",
                b"glClearColor\0",
                b"glCompileShader\0",
                b"glCreateProgram\0",
                b"glCreateShader\0",
                b"glDrawArrays\0",
                b"glEnable\0",
                b"glEnableVertexAttribArray\0",
                b"glGenBuffers\0",
                b"glGenTextures\0",
                b"glGenVertexArrays\0",
                b"glGetAttribLocation\0",
                b"glGetError\0",
                b"glGetIntegerv\0",
                b"glGetProgramInfoLog\0",
                b"glGetProgramiv\0",
                b"glGetShaderInfoLog\0",
                b"glGetShaderiv\0",
                b"glGetStringi\0",
                b"glGetUniformLocation\0",
                b"glLinkProgram\0",
                b"glShaderSource\0",
                b"glTexImage2D\0",
                b"glTexParameteri\0",
                b"glUniform1i\0",
                b"glUniform4fv\0",
                b"glUseProgram\0",
                b"glVertexAttribPointer\0",
            ],
        )?;
        procs[33].write(glDebugMessageCallbackARB_null_imp as *const ());
        let procs = unsafe { transmute(procs) };
        #[allow(unused_mut)]
        let mut ret = Procs {
            procs,
            has_ARB_debug_output: false,
        };
        let disabled_extensions = std::env::var("GL_DISABLED_EXTENSIONS");
        let disabled_extensions = disabled_extensions
            .as_ref()
            .map(|x| x.as_bytes())
            .unwrap_or(b"");
        let disabled_extensions =
            build_disabled_extension_list(disabled_extensions);
        let mut num_extensions = 0;
        unsafe { ret.GetIntegerv(GL_NUM_EXTENSIONS, &mut num_extensions) };
        for i in 0..num_extensions as GLuint {
            let ext = unsafe {
                CStr::from_ptr(transmute(ret.GetStringi(GL_EXTENSIONS, i)))
            }
            .to_bytes();
            if disabled_extensions.contains(ext) {
                continue;
            }
            match ext {
                b"GL_ARB_debug_output" => ret.has_ARB_debug_output = true,
                _ => (),
            }
        }
        if ret.has_ARB_debug_output {
            Procs::getprocs(
                &get_proc,
                unsafe { transmute(&mut ret.procs[33..34]) },
                &[b"glDebugMessageCallbackARB\0"],
            )?;
        }
        Ok(ret)
    }
    fn getprocs<E, F: Fn(&[u8]) -> Result<*const (), E>>(
        get_proc: &F,
        range: &mut [MaybeUninit<*const ()>],
        names: &[&[u8]],
    ) -> Result<(), E> {
        debug_assert_eq!(range.len(), names.len());
        for i in 0..range.len() {
            range[i].write(unsafe { transmute(get_proc(names[i])?) });
        }
        Ok(())
    }
    #[inline(always)]
    pub unsafe fn AttachShader(&self, program: GLuint, shader: GLuint) {
        (transmute::<_, extern "C" fn(program: GLuint, shader: GLuint)>(
            self.procs[0],
        ))(program, shader)
    }
    #[inline(always)]
    pub unsafe fn BindBuffer(&self, target: GLenum, buffer: GLuint) {
        (transmute::<_, extern "C" fn(target: GLenum, buffer: GLuint)>(
            self.procs[1],
        ))(target, buffer)
    }
    #[inline(always)]
    pub unsafe fn BindTexture(&self, target: GLenum, texture: GLuint) {
        (transmute::<_, extern "C" fn(target: GLenum, texture: GLuint)>(
            self.procs[2],
        ))(target, texture)
    }
    #[inline(always)]
    pub unsafe fn BindVertexArray(&self, array: GLuint) {
        (transmute::<_, extern "C" fn(array: GLuint)>(self.procs[3]))(array)
    }
    #[inline(always)]
    pub unsafe fn BufferData(
        &self,
        target: GLenum,
        size: GLsizeiptr,
        data: *const libc::c_void,
        usage: GLenum,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                target: GLenum,
                size: GLsizeiptr,
                data: *const libc::c_void,
                usage: GLenum,
            ),
        >(self.procs[4]))(target, size, data, usage)
    }
    #[inline(always)]
    pub unsafe fn Clear(&self, mask: GLbitfield) {
        (transmute::<_, extern "C" fn(mask: GLbitfield)>(self.procs[5]))(mask)
    }
    #[inline(always)]
    pub unsafe fn ClearColor(
        &self,
        red: GLfloat,
        green: GLfloat,
        blue: GLfloat,
        alpha: GLfloat,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                red: GLfloat,
                green: GLfloat,
                blue: GLfloat,
                alpha: GLfloat,
            ),
        >(self.procs[6]))(red, green, blue, alpha)
    }
    #[inline(always)]
    pub unsafe fn CompileShader(&self, shader: GLuint) {
        (transmute::<_, extern "C" fn(shader: GLuint)>(self.procs[7]))(shader)
    }
    #[inline(always)]
    pub unsafe fn CreateProgram(&self) -> GLuint {
        (transmute::<_, extern "C" fn() -> GLuint>(self.procs[8]))()
    }
    #[inline(always)]
    pub unsafe fn CreateShader(&self, r#type: GLenum) -> GLuint {
        (transmute::<_, extern "C" fn(r#type: GLenum) -> GLuint>(
            self.procs[9],
        ))(r#type)
    }
    #[inline(always)]
    pub unsafe fn DebugMessageCallbackARB(
        &self,
        callback: GLDEBUGPROCARB,
        userParam: *const libc::c_void,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                callback: GLDEBUGPROCARB,
                userParam: *const libc::c_void,
            ),
        >(self.procs[33]))(callback, userParam)
    }
    #[inline(always)]
    pub unsafe fn DrawArrays(
        &self,
        mode: GLenum,
        first: GLint,
        count: GLsizei,
    ) {
        (transmute::<
            _,
            extern "C" fn(mode: GLenum, first: GLint, count: GLsizei),
        >(self.procs[10]))(mode, first, count)
    }
    #[inline(always)]
    pub unsafe fn Enable(&self, cap: GLenum) {
        (transmute::<_, extern "C" fn(cap: GLenum)>(self.procs[11]))(cap)
    }
    #[inline(always)]
    pub unsafe fn EnableVertexAttribArray(&self, index: GLuint) {
        (transmute::<_, extern "C" fn(index: GLuint)>(self.procs[12]))(index)
    }
    #[inline(always)]
    pub unsafe fn GenBuffers(&self, n: GLsizei, buffers: *mut GLuint) {
        (transmute::<_, extern "C" fn(n: GLsizei, buffers: *mut GLuint)>(
            self.procs[13],
        ))(n, buffers)
    }
    #[inline(always)]
    pub unsafe fn GenTextures(&self, n: GLsizei, textures: *mut GLuint) {
        (transmute::<_, extern "C" fn(n: GLsizei, textures: *mut GLuint)>(
            self.procs[14],
        ))(n, textures)
    }
    #[inline(always)]
    pub unsafe fn GenVertexArrays(&self, n: GLsizei, arrays: *mut GLuint) {
        (transmute::<_, extern "C" fn(n: GLsizei, arrays: *mut GLuint)>(
            self.procs[15],
        ))(n, arrays)
    }
    #[inline(always)]
    pub unsafe fn GetAttribLocation(
        &self,
        program: GLuint,
        name: *const GLchar,
    ) -> GLint {
        (transmute::<
            _,
            extern "C" fn(program: GLuint, name: *const GLchar) -> GLint,
        >(self.procs[16]))(program, name)
    }
    #[inline(always)]
    pub unsafe fn GetError(&self) -> GLenum {
        (transmute::<_, extern "C" fn() -> GLenum>(self.procs[17]))()
    }
    #[inline(always)]
    pub unsafe fn GetIntegerv(&self, pname: GLenum, data: *mut GLint) {
        (transmute::<_, extern "C" fn(pname: GLenum, data: *mut GLint)>(
            self.procs[18],
        ))(pname, data)
    }
    #[inline(always)]
    pub unsafe fn GetProgramInfoLog(
        &self,
        program: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        infoLog: *mut GLchar,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                program: GLuint,
                bufSize: GLsizei,
                length: *mut GLsizei,
                infoLog: *mut GLchar,
            ),
        >(self.procs[19]))(program, bufSize, length, infoLog)
    }
    #[inline(always)]
    pub unsafe fn GetProgramiv(
        &self,
        program: GLuint,
        pname: GLenum,
        params: *mut GLint,
    ) {
        (transmute::<
            _,
            extern "C" fn(program: GLuint, pname: GLenum, params: *mut GLint),
        >(self.procs[20]))(program, pname, params)
    }
    #[inline(always)]
    pub unsafe fn GetShaderInfoLog(
        &self,
        shader: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        infoLog: *mut GLchar,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                shader: GLuint,
                bufSize: GLsizei,
                length: *mut GLsizei,
                infoLog: *mut GLchar,
            ),
        >(self.procs[21]))(shader, bufSize, length, infoLog)
    }
    #[inline(always)]
    pub unsafe fn GetShaderiv(
        &self,
        shader: GLuint,
        pname: GLenum,
        params: *mut GLint,
    ) {
        (transmute::<
            _,
            extern "C" fn(shader: GLuint, pname: GLenum, params: *mut GLint),
        >(self.procs[22]))(shader, pname, params)
    }
    #[inline(always)]
    pub unsafe fn GetStringi(
        &self,
        name: GLenum,
        index: GLuint,
    ) -> *const GLubyte {
        (transmute::<
            _,
            extern "C" fn(name: GLenum, index: GLuint) -> *const GLubyte,
        >(self.procs[23]))(name, index)
    }
    #[inline(always)]
    pub unsafe fn GetUniformLocation(
        &self,
        program: GLuint,
        name: *const GLchar,
    ) -> GLint {
        (transmute::<
            _,
            extern "C" fn(program: GLuint, name: *const GLchar) -> GLint,
        >(self.procs[24]))(program, name)
    }
    #[inline(always)]
    pub unsafe fn LinkProgram(&self, program: GLuint) {
        (transmute::<_, extern "C" fn(program: GLuint)>(self.procs[25]))(
            program,
        )
    }
    #[inline(always)]
    pub unsafe fn ShaderSource(
        &self,
        shader: GLuint,
        count: GLsizei,
        string: *mut *const GLchar,
        length: *const GLint,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                shader: GLuint,
                count: GLsizei,
                string: *mut *const GLchar,
                length: *const GLint,
            ),
        >(self.procs[26]))(shader, count, string, length)
    }
    #[inline(always)]
    pub unsafe fn TexImage2D(
        &self,
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        r#type: GLenum,
        pixels: *const libc::c_void,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                target: GLenum,
                level: GLint,
                internalformat: GLint,
                width: GLsizei,
                height: GLsizei,
                border: GLint,
                format: GLenum,
                r#type: GLenum,
                pixels: *const libc::c_void,
            ),
        >(self.procs[27]))(
            target,
            level,
            internalformat,
            width,
            height,
            border,
            format,
            r#type,
            pixels,
        )
    }
    #[inline(always)]
    pub unsafe fn TexParameteri(
        &self,
        target: GLenum,
        pname: GLenum,
        param: GLint,
    ) {
        (transmute::<
            _,
            extern "C" fn(target: GLenum, pname: GLenum, param: GLint),
        >(self.procs[28]))(target, pname, param)
    }
    #[inline(always)]
    pub unsafe fn Uniform1i(&self, location: GLint, v0: GLint) {
        (transmute::<_, extern "C" fn(location: GLint, v0: GLint)>(
            self.procs[29],
        ))(location, v0)
    }
    #[inline(always)]
    pub unsafe fn Uniform4fv(
        &self,
        location: GLint,
        count: GLsizei,
        value: *const GLfloat,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                location: GLint,
                count: GLsizei,
                value: *const GLfloat,
            ),
        >(self.procs[30]))(location, count, value)
    }
    #[inline(always)]
    pub unsafe fn UseProgram(&self, program: GLuint) {
        (transmute::<_, extern "C" fn(program: GLuint)>(self.procs[31]))(
            program,
        )
    }
    #[inline(always)]
    pub unsafe fn VertexAttribPointer(
        &self,
        index: GLuint,
        size: GLint,
        r#type: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const libc::c_void,
    ) {
        (transmute::<
            _,
            extern "C" fn(
                index: GLuint,
                size: GLint,
                r#type: GLenum,
                normalized: GLboolean,
                stride: GLsizei,
                pointer: *const libc::c_void,
            ),
        >(self.procs[32]))(
            index, size, r#type, normalized, stride, pointer
        )
    }
}

fn build_disabled_extension_list(
    disabled_extensions: &[u8],
) -> std::collections::HashSet<&[u8]> {
    disabled_extensions
        .split(|&x| {
            !((x >= b'0' && x <= b'9')
                || (x >= b'A' && x <= b'Z')
                || (x >= b'a' && x <= b'z')
                || (x == b'_'))
        })
        .filter_map(|x| match x {
            b"" => None,
            x => Some(x),
        })
        .collect()
}
