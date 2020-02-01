use web_sys::WebGlBuffer;
use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;

use wasmuri_core::*;

use super::shaders::TextProgram;
use super::Font;

use std::rc::Rc;

/// Instances of TextModel can be used to draw text on their webgl context. They can be created with the create_text_model
/// method of Font's.
/// 
/// To use an instance of TextModel, call its render method and read its description to see what all the parameters are for.
pub struct TextModel {

    font: Rc<Font>,

    buffer: WebGlBuffer,

    vertex_count: i32,
    total_width: f32
}

impl TextModel {

    pub(super) fn new(font: Rc<Font>, buffer: WebGlBuffer, char_count: usize, total_width: f32) -> TextModel {
        TextModel {
            font,
            buffer,
            vertex_count: (char_count * 6) as i32,
            total_width
        }
    }

    pub(super) fn bind(&self, shader_program: &TextProgram){
        let gl = &self.get_font().gl;

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));

        let num_components = 2;

        gl.vertex_attrib_pointer_with_i32(shader_program.get_relative_position() as u32, num_components, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(shader_program.get_relative_position() as u32);

        let f32_size = 4;
        gl.vertex_attrib_pointer_with_i32(shader_program.get_texture_coords() as u32, num_components, WebGlRenderingContext::FLOAT, false, 0, f32_size * num_components * self.vertex_count);
        gl.enable_vertex_attrib_array(shader_program.get_texture_coords() as u32);
    }

    /// Renders this TextModel at the given position with the given size and colors. The start_rendering
    /// method of the TextRenderer that created the font that created this TextModel should be called before 
    /// calling this method.
    /// 
    /// The first 3 parameters will determine the space that will be affected by the drawn text and its background. I will
    /// call the entire space that will be affected the 'render space'. The entire render space will be filled with the
    /// background color and the text will be drawn within the render space. The render space will be expressed in the
    /// OpenGL coordinate system, so the bottom-left corner would be (-1.0, -1.0) and the top-right corder would be
    /// (1.0, 1.0).
    /// 
    /// Note that only characters like Á will actually (almost) touch the top of the render space and only characters like 
    /// 'y' will (almost) touch the bottom of the render space.
    /// 
    /// The parameters offset_x and offset_y determine the bottom-left corner of the render space.
    /// 
    /// The scale_y parameter determines the height of the render space (in OpenGL coordinate space), so a scale_y of 2.0 with 
    /// an offset_y of -1.0 would claim the full height of the canvas. The width of the text will depend on both the width of
    /// the string and scale_y. You can find the width in advance using the get_render_width method of this TextModel.
    /// 
    /// The fill_color will determine the color of the interior of the rendered text. If you make it transparent, you will see
    /// the background_color instead.
    /// 
    /// The stroke_color will determine the color of the lines at the borders of the rendered text. If the Font was created
    /// with a line_width of 0, the stroke_color won't have any effect. Otherwise, the stroke_color will have effect. If the
    /// stroke_color is the same as the fill_color, the text will be rendered (a little) thicker. If the stroke_color is
    /// transparent, the text will be rendered (a little) thinner.
    /// 
    /// The background_color will determine the color of the render space wherever no text is drawn (or the text is (partially)
    /// transparent). If it is transparent, the text will be drawn over whatever the previous color was.
    pub fn render(&self, offset_x: f32, offset_y: f32, scale_y: f32, colors: TextColors){
        let need_set_font;
        let my_font = self.get_font();
        {
            let selected_font = my_font.selected_font.get();
            match selected_font {
                Some(font_id) => need_set_font = font_id != my_font.id,
                None => need_set_font = true
            };
        }

        if need_set_font {
            my_font.set_current();
            my_font.selected_font.set(Some(my_font.id));
        }

        let scale_x = scale_y / my_font.aspect_ratio.get();

        let mut shader = my_font.shader_program.borrow_mut();
        shader.set_background_color(colors.background_color);
        shader.set_fill_color(colors.fill_color);
        shader.set_stroke_color(colors.stroke_color);
        shader.set_screen_position(offset_x, offset_y);
        shader.set_scale(scale_x, scale_y);
        self.bind(&shader);
        my_font.gl.draw_arrays(GL::TRIANGLES, 0, self.vertex_count);
    }

    /// This method can be used to predict the width of the text drawn with the render method.
    /// 
    /// The scale_y parameter should be the same as the scale_y you are planning to pass to the render method.
    /// 
    /// The result of this method will be given in the OpenGL coordinate space, so a return value of 2.0 
    /// means the text would span the entire canvas width (if the offset_x would be -1.0).
    pub fn get_render_width(&self, scale_y: f32) -> f32 {
        let my_font = self.get_font();
        let scale_x = scale_y / my_font.aspect_ratio.get();
        scale_x * self.total_width
    }

    pub fn get_font(&self) -> &Rc<Font> {
        &self.font
    }
}

impl Drop for TextModel {

    fn drop(&mut self){
        self.get_font().gl.delete_buffer(Some(&self.buffer));
    }
}