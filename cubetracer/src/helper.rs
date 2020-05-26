extern crate gl;
extern crate image;

use crate::errors::*;
use crate::{glchk_expr, glchk_stmt, ConfigurableShader};

use std::ffi::{c_void, CString};
use std::{mem, ptr};

use gl::types::*;

pub fn load_texture(i: u32, path: &std::path::Path) -> Result<u32, GLError> {
    let image = image::open(path).expect("can't load texture").into_rgba();
    let mut tex_out = 0;

    glchk_stmt!(
        gl::GenTextures(2, &mut tex_out);
        gl::ActiveTexture(gl::TEXTURE0 + i);
        gl::BindTexture(gl::TEXTURE_2D, tex_out);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            image.width() as i32,
            image.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            image.into_raw().as_ptr() as *const _ as *const c_void,
        );
    );

    Ok(tex_out)
}

pub fn generate_texture(width: u32, height: u32) -> Result<u32, GLError> {
    let mut tex_out = 0;

    glchk_stmt!(
        gl::GenTextures(1, &mut tex_out);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, tex_out);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA32F as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::FLOAT,
            ptr::null(),
        );

        gl::BindImageTexture(0, tex_out, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F);
    );

    Ok(tex_out)
}

pub fn get_uniform_location(program: u32, var_name: &str) -> Result<i32, GLError> {
    let c_var_name = CString::new(var_name).unwrap();
    let loc = glchk_expr!(gl::GetUniformLocation(program, c_var_name.as_ptr()));
    if loc == -1 {
        Err(GLError::UniformNotFound {
            name: var_name.to_string(),
        })
    } else {
        Ok(loc)
    }
}

pub fn get_ssbo_location(program: u32, var_name: &str) -> Result<i32, GLError> {
    let c_var_name = CString::new(var_name).unwrap();
    let loc = glchk_expr!(gl::GetProgramResourceIndex(
        program,
        gl::SHADER_STORAGE_BLOCK,
        c_var_name.as_ptr()
    )) as i32;

    if loc == -1 {
        Err(GLError::UniformNotFound {
            name: var_name.to_string(),
        })
    } else {
        Ok(loc)
    }
}
pub fn build_program_raytracer(view_size: usize, shadow_activated: bool) -> Result<u32, GLError> {
    let mut shader_compute = ConfigurableShader::new(include_str!("../shaders/raytracer.comp"));
    shader_compute.var("CST_VIEW_DISTANCE", view_size);
    shader_compute.var("CST_SHADOW_ACTIVATED", shadow_activated);

    let shader_compute = shader_compute.build(gl::COMPUTE_SHADER)?;

    let program = glchk_expr!(gl::CreateProgram());
    glchk_stmt!(
        gl::AttachShader(program, shader_compute);
        gl::LinkProgram(program);
    );

    gl_check_error_program(program, gl::LINK_STATUS)
}

pub fn _update_ssbo_data<T>(ssbo: u32, data: &[T]) -> Result<(), GLError> {
    glchk_stmt!(gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo););

    let dst: *mut c_void = glchk_expr!(gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY));

    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr() as *const c_void, dst, data.len());
    }

    let unmapped = glchk_expr!(gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER));
    assert!(unmapped > 0);

    glchk_stmt!(
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
    );

    Ok(())
}

pub fn _update_ssbo<T>(ssbo: u32, data: &Vec<T>) -> Result<(), GLError> {
    glchk_stmt!(
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
        gl::BufferData(
            gl::SHADER_STORAGE_BUFFER,
            (data.len() * mem::size_of::<T>()) as GLsizeiptr,
            data.as_ptr() as *const c_void,
            gl::STREAM_DRAW,
        );
    );
    Ok(())
}

pub fn update_ssbo_partial<T>(ssbo: u32, offset: usize, data: &Vec<T>) -> Result<(), GLError> {
    glchk_stmt!(
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
        gl::BufferSubData(
            gl::SHADER_STORAGE_BUFFER,
            offset as GLintptr,
            (data.len() * mem::size_of::<T>()) as GLsizeiptr,
            data.as_ptr() as *const c_void
        );
    );

    Ok(())
}

pub fn make_ssbo(program: u32, var_name: &str, size: usize) -> Result<u32, GLError> {
    let mut ssbo = 0;

    glchk_stmt!(
        gl::GenBuffers(1, &mut ssbo);
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
        gl::BufferData(
            gl::SHADER_STORAGE_BUFFER,
            size as GLsizeiptr,
            ptr::null(),
            gl::STREAM_DRAW,
        );
        gl::BindBufferBase(
            gl::SHADER_STORAGE_BUFFER,
            get_ssbo_location(program, var_name)? as u32,
            ssbo
        );
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
    );

    Ok(ssbo)
}

pub fn make_quad_vao(program: u32) -> Result<u32, GLError> {
    let vertices: [f32; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0];

    let (mut vbo, mut vao) = (0, 0);

    glchk_stmt!(
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
    );

    let c_var_name_pos = CString::new("in_pos").unwrap();
    let attr_pos = glchk_expr!(gl::GetAttribLocation(program, c_var_name_pos.as_ptr()) as u32);

    glchk_stmt!(
        gl::VertexAttribPointer(attr_pos, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(attr_pos);
    );

    Ok(vao)
}

pub fn build_program_quad() -> Result<u32, GLError> {
    let program = glchk_expr!(gl::CreateProgram());

    let shader_vertex =
        ConfigurableShader::new(include_str!("../shaders/vertex.glsl")).build(gl::VERTEX_SHADER)?;

    glchk_stmt!(
        gl::AttachShader(program, shader_vertex);
    );

    let shader_fragment = ConfigurableShader::new(include_str!("../shaders/fragment.glsl"))
        .build(gl::FRAGMENT_SHADER)?;

    glchk_stmt!(
        gl::AttachShader(program, shader_fragment);
        gl::LinkProgram(program);
    );
    gl_check_error_program(program, gl::LINK_STATUS)?;

    glchk_stmt!(
        gl::DeleteShader(shader_vertex);
        gl::DeleteShader(shader_fragment);
        gl::ProgramUniform1i(
            program,
            gl::GetUniformLocation(program, CString::new("uni_text").unwrap().as_ptr()),
            0,
        );


    );

    Ok(program)
}
