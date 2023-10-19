use std::{fmt::Write as FmtWrite, io::Write, mem::transmute};

use sdl2::video::{GLProfile, WindowBuilder};

use anyhow::{anyhow, Context};
use log::*;

mod gl31;
use gl31::*;
mod glerr;

use super::*;

const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("fragment.glsl");
const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("vertex.glsl");

/// A simulated framebuffer display.
pub struct Display {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    gl: Procs,
    screen_texture: GLuint,
    program: GLuint,
}

impl Display {
    pub fn new(
        video: &sdl2::VideoSubsystem,
        window_builder: impl Fn() -> WindowBuilder,
    ) -> anyhow::Result<Display> {
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 1);
        #[cfg(debug_assertions)]
        gl_attr.set_context_flags().debug();
        gl_attr.set_multisample_buffers(0);
        gl_attr.set_multisample_samples(0);
        gl_attr.set_depth_size(0);
        gl_attr.set_framebuffer_srgb_compatible(true);
        let window = window_builder()
            .allow_highdpi()
            .opengl()
            .build()
            .context("unable to create window for OpenGL context")?;
        let gl_context = window.gl_create_context().map_err(|x| {
            anyhow!("unable to create OpenGL 3.1 context: {x}")
        })?;
        let gl = Procs::new(|proc| {
            // Unsafe justification: the input is known to be a static,
            // null-terminated string.
            let ret = unsafe {
                sdl2_sys::SDL_GL_GetProcAddress(transmute(proc.as_ptr()))
            };
            if ret.is_null() {
                Err(anyhow!(
                    "Unable to find the procedure named {}: {}",
                    String::from_utf8_lossy(&proc[..proc.len() - 1]),
                    sdl2::get_error()
                ))
            } else {
                // Unsafe justification: a non-null return address is a valid
                // OpenGL procedure entry point.
                Ok(unsafe { transmute(ret) })
            }
        })?;
        let mut screen_texture = 0;
        let program;
        // unsafe justification: all OpenGL interfacing is unsafe :(
        unsafe {
            // do fancy debug output if we can
            if gl.has_ARB_debug_output {
                debug!(
                    "ARB_debug_output extension is present. OpenGL errors \
                    will be detected promptly."
                );
                gl.Enable(0x92E0); // Mesa bug workaround
                gl.GetError(); // ?!
                #[cfg(debug_assertions)]
                gl.Enable(GL_DEBUG_OUTPUT_SYNCHRONOUS_ARB);
                gl.DebugMessageCallbackARB(
                    Some(debug_callback),
                    std::ptr::null(),
                );
            } else {
                info!(
                    "ARB_debug_output extension is missing. OpenGL errors may \
                   not be detected promptly."
                );
            }
            gl.Enable(GL_FRAMEBUFFER_SRGB);
            gl.GenTextures(1, &mut screen_texture);
            gl.BindTexture(GL_TEXTURE_2D, screen_texture);
            gl.TexParameteri(
                GL_TEXTURE_2D,
                GL_TEXTURE_MIN_FILTER,
                GL_NEAREST as GLint,
            );
            gl.TexParameteri(
                GL_TEXTURE_2D,
                GL_TEXTURE_MAG_FILTER,
                GL_NEAREST as GLint,
            );
            let shader_fragment = compile_shader(
                &gl,
                "fragment shader",
                GL_FRAGMENT_SHADER,
                &[FRAGMENT_SHADER_SOURCE],
            )?;
            let shader_vertex = compile_shader(
                &gl,
                "vertex shader",
                GL_VERTEX_SHADER,
                &[VERTEX_SHADER_SOURCE],
            )?;
            program = link_program(
                &gl,
                "the one, the only",
                &[shader_fragment, shader_vertex],
            )?;
            gl.UseProgram(program);
            let mut vao = 0;
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);
            let mut vbo = 0;
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(GL_ARRAY_BUFFER, vbo);
            setup_attribs(
                &gl,
                program,
                "the one, the only",
                &[
                    (b"pos\0", &|gl, loc| {
                        gl.VertexAttribPointer(
                            loc,
                            2,
                            GL_FLOAT,
                            0,
                            16,
                            std::ptr::null(),
                        )
                    }),
                    (b"vert_uv\0", &|gl, loc| {
                        gl.VertexAttribPointer(
                            loc,
                            2,
                            GL_FLOAT,
                            0,
                            16,
                            8usize as *const libc::c_void,
                        )
                    }),
                ],
            );
            assertgl(&gl, "while initializing GL state");
        }
        Ok(Display {
            window,
            gl,
            gl_context,
            screen_texture,
            program,
        })
    }
    pub fn update(
        &mut self,
        bits: &Bitmap,
        pixel_aspect_ratio: f32,
        margin_color: &[f32; 4],
        zero_color: &[f32; 4],
        one_color: &[f32; 4],
        dirty_region: Option<Rectangle>,
    ) -> anyhow::Result<()> {
        self.window.gl_make_current(&self.gl_context).map_err(|x| {
            anyhow!("unable to make OpenGL context current: {x}")
        })?;
        assertgl(&self.gl, "before rendering (error leaked!)");
        let screen_bounds = Rectangle {
            left: 0,
            top: 0,
            right: bits.width as i32,
            bottom: bits.height as i32,
        };
        let dirty_region = dirty_region
            .unwrap_or(screen_bounds)
            .expand_by(1)
            .intersection(screen_bounds);
        // TODO: account for double, possibly triple buffering
        let (window_width, window_height) = self.window.drawable_size();
        let window_width = window_width as f32;
        let window_height = window_height as f32 * pixel_aspect_ratio;
        let bitmap_width = bits.width as f32;
        let bitmap_height = bits.height as f32;
        let (x_margins, y_margins, _scale_factor) = calculate_mapping(
            window_width,
            window_height,
            bitmap_width,
            bitmap_height,
        );
        // we don't need to multiply by 0.5 because it cancels out from a
        // multiply by 2 we would do later
        let left_margin = x_margins / window_width;
        let top_margin = y_margins / window_height;
        let u_left = dirty_region.left as f32;
        let u_right = dirty_region.right as f32;
        let v_top = dirty_region.top as f32;
        let v_bottom = dirty_region.bottom as f32;
        let x_left = -1.0 + left_margin;
        let x_right = 1.0 - left_margin;
        let y_top = 1.0 - top_margin;
        let y_bottom = -1.0 + top_margin;
        let buffer = [
            x_left, y_bottom, u_left, v_bottom, //
            x_right, y_bottom, u_right, v_bottom, //
            x_left, y_top, u_left, v_top, //
            x_right, y_top, u_right, v_top,
        ];
        unsafe {
            self.gl.BindTexture(GL_TEXTURE_2D, self.screen_texture);
            self.gl.TexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_R32UI as GLint,
                bits.pitch_words as GLint,
                bits.height as GLint,
                0,
                GL_RED_INTEGER,
                GL_UNSIGNED_INT,
                &bits.bits[0] as *const BitmapWord as *const GLvoid,
            );
            assertgl(&self.gl, "uploading texture");
            self.gl.BufferData(
                GL_ARRAY_BUFFER,
                64,
                &buffer[0] as *const f32 as *const GLvoid,
                GL_STREAM_DRAW,
            );
            setup_uniforms(
                &self.gl,
                self.program,
                "the one, the only",
                &[
                    (b"bits\0", &|gl, loc| gl.Uniform1i(loc, 0)),
                    (b"zerocolor\0", &|gl, loc| {
                        gl.Uniform4fv(loc, 1, &zero_color[0])
                    }),
                    (b"onecolor\0", &|gl, loc| {
                        gl.Uniform4fv(loc, 1, &one_color[0])
                    }),
                ],
            );
            assertgl(&self.gl, "setting up shader");
            if (left_margin != 0.0 || top_margin != 0.0)
                && dirty_region == screen_bounds
            {
                self.gl.ClearColor(
                    margin_color[0],
                    margin_color[1],
                    margin_color[2],
                    margin_color[3],
                );
                self.gl.Clear(GL_COLOR_BUFFER_BIT);
            }
            self.gl.DrawArrays(GL_TRIANGLE_STRIP, 0, 4);
        }
        assertgl(&self.gl, "after rendering");
        self.window.gl_swap_window();
        Ok(())
    }
    pub fn get_window(&self) -> &sdl2::video::Window {
        &self.window
    }
    pub fn real_coords_to_virtual(
        &self,
        x: i32,
        y: i32,
        bits_width: u32,
        bits_height: u32,
        pixel_aspect_ratio: f32,
    ) -> (i32, i32) {
        let (window_width, window_height) = self.window.size();
        let window_width = window_width as f32;
        let window_height = window_height as f32 * pixel_aspect_ratio;
        let bits_width = bits_width as f32;
        let bits_height = bits_height as f32;
        let (x_margins, y_margins, scale_factor) = calculate_mapping(
            window_width,
            window_height,
            bits_width,
            bits_height,
        );
        let x = (x as f32) - (x_margins * 0.5);
        let y = (y as f32 * pixel_aspect_ratio) - (y_margins * 0.5);
        (
            (x / scale_factor).floor() as i32,
            (y / scale_factor).floor() as i32,
        )
    }
}

// Utilities shamelessly stolen from Vectoracious:

/// Check for OpenGL errors. If there were any, complain.
fn assertgl(gl: &Procs, wo: &str) {
    let mut errors = vec![];
    'outer: loop {
        // Unsafe justification: glGetError is safe to call.
        let e = unsafe { gl.GetError() };
        if e == 0 {
            break;
        }
        for &(code, name) in glerr::ERROR_TABLE.iter() {
            if code == e {
                errors.push(name);
                continue 'outer;
            }
        }
        errors.push("unknown");
    }
    if !errors.is_empty() {
        if cfg!(debug_assertions) {
            panic!("OpenGL errors were detected {}: {:?}", wo, errors)
        } else {
            warn!("OpenGL errors were detected {}: {:?}", wo, errors)
        }
    }
}

extern "C" fn debug_callback(
    source: GLenum,
    typ: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    _: *const GLvoid,
) {
    let source = match source {
        GL_DEBUG_SOURCE_API_ARB => "OpenGL".to_owned(),
        GL_DEBUG_SOURCE_APPLICATION_ARB => "the user(?!)".to_owned(),
        GL_DEBUG_SOURCE_SHADER_COMPILER_ARB => {
            "the shader compiler".to_owned()
        }
        GL_DEBUG_SOURCE_WINDOW_SYSTEM_ARB => "the window system".to_owned(),
        GL_DEBUG_SOURCE_THIRD_PARTY_ARB => "a third party".to_owned(),
        GL_DEBUG_SOURCE_OTHER_ARB => "\"other\"(!?)".to_owned(),
        x => format!("(unknown 0x{:x})", x),
    };
    let typ = match typ {
        GL_DEBUG_TYPE_ERROR_ARB => "an error".to_owned(),
        GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR_ARB => {
            "deprecated behavior".to_owned()
        }
        GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR_ARB => {
            "undefined behavior".to_owned()
        }
        GL_DEBUG_TYPE_PORTABILITY_ARB => "unportable usage".to_owned(),
        GL_DEBUG_TYPE_PERFORMANCE_ARB => "poorly-performing usage".to_owned(),
        GL_DEBUG_TYPE_OTHER_ARB => {
            if severity <= GL_DEBUG_SEVERITY_LOW_ARB {
                "something boring".to_owned()
            } else {
                "something spooky".to_owned()
            }
        }
        x => format!("(unknown 0x{:x})", x),
    };
    let message = unsafe {
        std::slice::from_raw_parts(message as *const u8, length as usize)
    };
    let message = String::from_utf8_lossy(message);
    let message = message.strip_suffix('\n').unwrap_or(&message);
    match severity {
        GL_DEBUG_SEVERITY_HIGH_ARB => {
            error!("{} detected {}: [HIGH, {}] {}", source, typ, id, message);
        }
        GL_DEBUG_SEVERITY_MEDIUM_ARB => {
            warn!("{} detected {}: [MEDIUM, {}] {}", source, typ, id, message);
        }
        GL_DEBUG_SEVERITY_LOW_ARB => {
            info!("{} detected {}: [LOW, {}] {}", source, typ, id, message);
        }
        _ => {
            debug!("{} detected {}: [???, {}] {}", source, typ, id, message);
        }
    }
    if std::env::var_os("RUST_BACKTRACE").is_some() {
        let mut backtrace = String::new();
        backtrace::trace(|frame| {
            backtrace::resolve_frame(frame, |res| {
                // @%!@#Q$&%
                let symbol = match res.name() {
                    Some(x) => format!("{}", x),
                    None => "???".to_owned(),
                };
                if symbol.starts_with("backtrace:")
                    || symbol.contains("::debug_callback")
                {
                    return;
                }
                let filename = match res.filename() {
                    Some(x) => x
                        .file_name()
                        .unwrap()
                        .to_os_string()
                        .into_string()
                        .unwrap(),
                    None => "???".to_owned(),
                };
                let lineno = res.lineno().unwrap_or(0);
                writeln!(backtrace, "{} at {}:{}", symbol, filename, lineno)
                    .unwrap();
            });
            true
        });
        trace!("backtrace:\n{}", backtrace);
    }
}

fn compile_shader(
    gl: &Procs,
    wat: &str,
    typ: GLenum,
    texts: &[&[u8]],
) -> anyhow::Result<GLuint> {
    const SHADER_VERSION_SUPPLEMENT: &[u8] = br#"
#version 140
"#;
    // Unsafe justification: Lots of GL calls. We're careful about length but
    // we do assume the GL implementation doesn't lie to us in ways that are
    // TECHNICALLY allowed by the standard.
    unsafe {
        let shader = gl.CreateShader(typ);
        // if we get more than 9 source elements, this won't be enough,
        // but screw that
        let needed_len = texts
            .iter()
            .fold(SHADER_VERSION_SUPPLEMENT.len(), |a, x| a + x.len() + 11);
        let mut buf: Vec<u8> = Vec::with_capacity(needed_len);
        buf.extend_from_slice(SHADER_VERSION_SUPPLEMENT);
        for (n, &text) in texts.iter().enumerate() {
            writeln!(buf, "#line 0 {}", n).unwrap();
            buf.write_all(text).unwrap();
            buf.write_all(b"\n").unwrap();
        }
        gl.ShaderSource(
            shader,
            1,
            transmute(&&buf[0]),
            [buf.len() as GLint].as_ptr(),
        );
        gl.CompileShader(shader);
        let mut status: GLint = 0;
        gl.GetShaderiv(shader, GL_COMPILE_STATUS, &mut status);
        let mut log_length: GLint = 0;
        gl.GetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut log_length);
        let mut info_log: Vec<u8> = vec![];
        assert!(log_length >= 0);
        info_log.clear();
        info_log.resize(log_length as usize, 0);
        if log_length > 1 {
            gl.GetShaderInfoLog(
                shader,
                log_length,
                std::ptr::null_mut(),
                &mut info_log[0] as *mut u8 as *mut i8,
            );
        }
        if status == 0 {
            Err(anyhow!(
                "Unable to compile {}!\n{}",
                wat,
                String::from_utf8_lossy(&info_log[..info_log.len() - 1])
            ))?
        } else if log_length > 1 {
            warn!(
                "Diagnostics were generated while compiling {}:\n{}",
                wat,
                String::from_utf8_lossy(&info_log[..info_log.len() - 1])
            );
        }
        assertgl(gl, "compiling a shader");
        Ok(shader)
    }
}

fn link_program(
    gl: &Procs,
    wat: &str,
    shaders: &[GLuint],
) -> anyhow::Result<GLuint> {
    // Unsafe justification: See `compile_shader`
    unsafe {
        let program = gl.CreateProgram();
        for &shader in shaders {
            gl.AttachShader(program, shader);
        }
        gl.LinkProgram(program);
        let mut status: GLint = 0;
        gl.GetProgramiv(program, GL_LINK_STATUS, &mut status);
        let mut log_length: GLint = 0;
        gl.GetProgramiv(program, GL_INFO_LOG_LENGTH, &mut log_length);
        let mut info_log: Vec<u8> = vec![];
        assert!(log_length >= 0);
        info_log.clear();
        info_log.resize(log_length as usize, 0);
        if log_length > 1 {
            gl.GetProgramInfoLog(
                program,
                log_length,
                std::ptr::null_mut(),
                &mut info_log[0] as *mut u8 as *mut i8,
            );
        }
        if status == 0 {
            Err(anyhow!(
                "Unable to link {}!\n{}",
                wat,
                String::from_utf8_lossy(&info_log[..info_log.len() - 1])
            ))?
        } else if log_length > 1 {
            warn!(
                "Diagnostics were generated while linking {}:\n{}",
                wat,
                String::from_utf8_lossy(&info_log[..info_log.len() - 1])
            );
        }
        assertgl(gl, "linking a shader program");
        Ok(program)
    }
}

#[allow(clippy::type_complexity)]
fn setup_attribs(
    gl: &Procs,
    program: GLuint,
    program_name: &str,
    attribs: &[(&[u8], &dyn Fn(&Procs, GLuint))],
) {
    unsafe {
        gl.UseProgram(program);
        for &(name, f) in attribs.iter() {
            debug_assert!(name.ends_with(&[0]));
            let loc = gl.GetAttribLocation(program, transmute(name.as_ptr()));
            if loc < 0 {
                warn!(
                    "couldn't find expected shader attribute {:?} \
                     in the {:?} program",
                    String::from_utf8_lossy(&name[..name.len() - 1]),
                    program_name
                );
            } else {
                let loc = loc as GLuint;
                gl.EnableVertexAttribArray(loc);
                f(gl, loc);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn setup_uniforms(
    gl: &Procs,
    program: GLuint,
    program_name: &str,
    uniforms: &[(&[u8], &dyn Fn(&Procs, GLint))],
) {
    unsafe {
        gl.UseProgram(program);
        for &(name, f) in uniforms.iter() {
            debug_assert!(name.ends_with(&[0]));
            let loc = gl.GetUniformLocation(program, transmute(name.as_ptr()));
            if loc < 0 {
                warn!(
                    "couldn't find expected shader uniform {:?} \
                     in the {:?} program",
                    String::from_utf8_lossy(&name[..name.len() - 1]),
                    program_name
                );
            } else {
                let loc = loc as GLint;
                f(gl, loc);
            }
        }
    }
}

fn calculate_mapping(
    window_width: f32,
    window_height: f32,
    bitmap_width: f32,
    bitmap_height: f32,
) -> (f32, f32, f32) {
    let (x_margins, y_margins, scale_factor);
    let scale_width = window_width / bitmap_width;
    let scale_height = window_height / bitmap_height;
    if scale_width > scale_height {
        scale_factor = scale_height;
        x_margins = window_width - bitmap_width * scale_factor;
        assert!(x_margins >= 0.0);
        y_margins = 0.0;
    } else {
        scale_factor = scale_width;
        y_margins = window_height - bitmap_height * scale_factor;
        assert!(y_margins >= 0.0);
        x_margins = 0.0;
    }
    (x_margins, y_margins, scale_factor)
}
