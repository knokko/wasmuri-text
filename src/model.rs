use web_sys::WebGlBuffer;
use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;

use super::shaders::TextProgram;

pub struct TextModel<'a> {

    gl: &'a WebGlRenderingContext,

    buffer: WebGlBuffer,

    vertex_count: i32,
    total_width: f32
}

impl<'a> TextModel<'a> {

    pub(super) fn new(gl: &'a WebGlRenderingContext, buffer: WebGlBuffer, char_count: usize, total_width: f32) -> TextModel<'a> {
        TextModel {
            gl,
            buffer,
            vertex_count: (char_count * 6) as i32,
            total_width
        }
    }

    pub(super) fn bind(&self, shader_program: &TextProgram){
        let gl = &self.gl;

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));

        let num_components = 2;

        gl.vertex_attrib_pointer_with_i32(shader_program.get_relative_position() as u32, num_components, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(shader_program.get_relative_position() as u32);

        let f32_size = 4;
        gl.vertex_attrib_pointer_with_i32(shader_program.get_texture_coords() as u32, num_components, WebGlRenderingContext::FLOAT, false, 0, f32_size * num_components * self.vertex_count);
        gl.enable_vertex_attrib_array(shader_program.get_texture_coords() as u32);
    }

    pub(super) fn get_vertex_count(&self) -> i32 {
        self.vertex_count
    }

    pub(super) fn get_width(&self) -> f32 {
        self.total_width
    }
}

impl<'a> Drop for TextModel<'a> {

    fn drop(&mut self){
        self.gl.delete_buffer(Some(&self.buffer));
    }
}