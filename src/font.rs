use web_sys::CanvasRenderingContext2d;
use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;
use web_sys::WebGlTexture;
use web_sys::window;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlElement;

use js_sys::Float32Array;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use wasmuri_core::util::print;

use std::cell::RefCell;
use std::cell::Cell;
use std::rc::Rc;

use super::character::Character;
use super::model::TextModel;
use super::shaders::TextProgram;

#[derive(PartialEq,Eq,Copy,Clone)]
pub(super) struct FontID {

    value: usize
}

impl FontID {

    pub(super) fn new(value: usize) -> FontID {
        FontID {
            value
        }
    }
}

#[derive(PartialEq,Eq,Clone)]
/// Instances of FontDetails represent properties of JavaScript canvas fonts, but without the font size.
/// An example of a JavaScript font is "bold 40px Arial". To obtain a FontDetails instance corresponding 
/// to that example font, you would need to use FontDetails::new("bold", "Arial").
/// 
/// Whenever a Font is created, an instance of FontDetails needs to be passed as parameter to describe all
/// the info/details about the font to create. Internally, a canvas with a 2d context will be used to generate
/// the backing texture of all characters for the Font. The before_size of the FontDetails (plus an extra whitespace)
/// will literally be pasted before the size declaration of the font and the after_size of the FontDetails will
/// be pasted after the size declaration (plus an extra whitespace). 
/// 
/// The size declaration of the font will be handled internally, but note that the size of the drawn text does
/// NOT depend on that because the scaling of rendered text will be done on-the-fly.
pub struct FontDetails {

    before_size: String,
    after_size: String
}

impl FontDetails {

    /// Create a new instance of FontDetails with the given before and after string. See the description of
    /// FontDetails for an explanation about these values.
    pub const fn from_string(before_size: String, after_size: String) -> FontDetails {
        FontDetails {
            before_size,
            after_size
        }
    }

    /// Create a new instance of FontDetails with the given before and after string. See the description of
    /// FontDetails for an explanation about these values.
    pub fn from_str(before_size: &str, after_size: &str) -> FontDetails {
        FontDetails {
            before_size: before_size.to_string(),
            after_size: after_size.to_string()
        }
    }

    /// Gets the part of the font string that should be placed before the size. See the description of FontDetails 
    /// for an explanation about the string value.
    pub fn get_before_size(&self) -> &str {
        &self.before_size
    }

    /// Gets the part of the font string that should be placed after the size. See the description of FontDetails 
    /// for an explanation about the string value.
    pub fn get_after_size(&self) -> &str {
        &self.after_size
    }
}

/// Fonts are the structs responsible for creating TextModel's that can draw text onto the webgl canvas. Instances
/// of Font can be created by using the add_font or add_fonts method of a TextRenderer.
/// 
/// There are 2 ways to obtain a Font from a TextRenderer:
/// 
/// -If you created the font with the add_font method of a TextRenderer, you can store the return value which
/// will be a reference to the created font.
/// 
/// -If you have the details of the font, you can use the get_font_by_details method of the TextRenderer that
/// created the font.
/// 
/// To use a Font, you can use the create_text_model method of the font. First use the create_text_model method 
/// to obtain a TextModel for the text you would like to render. Then call the render method of the TextModel to 
/// actually render the text. You are encouraged to store the result of create_text_model so that you can reuse it 
/// many times rather than creating it again and again.
pub struct Font {

    font_details: FontDetails,

    max_text_height: u32,
    pub(super) aspect_ratio: Cell<f32>,

    pub(super) id: FontID,
    pub(super) selected_font: Rc<RefCell<Option<FontID>>>,

    characters: Vec<Option<Character>>,

    pub(super) gl: Rc<WebGlRenderingContext>,
    pub(super) shader_program: Rc<RefCell<TextProgram>>,
    texture: WebGlTexture
}

impl Font {

    pub(super) fn new(gl: Rc<WebGlRenderingContext>, shader_program: Rc<RefCell<TextProgram>>, font_id: FontID, selected_font: Rc<RefCell<Option<FontID>>>, font_size: usize, line_width: f64, font_details: FontDetails, chars: &str) -> Font {
        let document = window().unwrap().document().unwrap();
        let font_string = &format!("{} {}px {}", font_details.get_before_size(), font_size, font_details.get_after_size());

        let test_canvas = document.create_element("canvas").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
        test_canvas.set_width(1);
        test_canvas.set_height(1);

        let test_ctx = test_canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
        test_ctx.set_font(font_string);

        // Even though chars.len() will return the length in bytes rather than the length in chars,
        // it is still a nice approximation and the initial capacity doesn't have to be exact.
        let mut char_sizes = Vec::with_capacity(chars.len());

        let mut max_height = 0;

        let line_margin = (2.0 * line_width * font_size as f64).ceil() as u32;

        let mut max_char_code = 0;
        let mut char_counter = 0;
        
        for character in chars.chars() {
            let mut substring = [0; 4];
            let bounds = test_ctx.measure_text(character.encode_utf8(&mut substring)).unwrap();

            // I would like to obtain stuff like height as well, but... well... browser compatibility...
            // https://developer.mozilla.org/en-US/docs/Web/API/TextMetrics
            let char_width = bounds.width().ceil() as u32;

            // So... let's obtain the char_height the hard way...
            // Code is based on https://github.com/knokko/Image-Helper/blob/master/ImageFactory.js -> determineFontHeight
            let body = document.body().unwrap();
            let dummy = document.create_element("div").unwrap().dyn_into::<HtmlElement>().unwrap();
            let dummy_text = document.create_text_node("M");
            dummy.append_child(&dummy_text).unwrap();
            dummy.set_attribute("style", &format!("font: {};", font_string)).unwrap();
            body.append_child(&dummy).unwrap();
            let char_height = dummy.offset_height() as u32;
            body.remove_child(&dummy).unwrap();

            char_sizes.push((char_width, char_height));

            if char_height > max_height {
                max_height = char_height;
            }

            let char_code = character as usize;
            if char_code > max_char_code {
                max_char_code = char_code;
            }

            char_counter += 1;
        }

        let chars_per_row = (char_counter as f64).sqrt().ceil() as u32;
        let rows;
        {
            let divided = char_counter / chars_per_row;
            if divided * chars_per_row >= char_counter {
                rows = divided;
            } else {
                rows = divided + 1;
            }
        }

        let total_width;
        {
            // We will have to start with some value...
            let mut max_width = 0;

            for row in char_sizes.chunks(chars_per_row as usize) {
                let mut current_width = 0;
                for char_size in row {
                    current_width += char_size.0 + 2 * line_margin;
                }
                if current_width > max_width {
                    max_width = current_width;
                }
            }

            total_width = max_width;
        }

        let texture_canvas = document.create_element("canvas").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
        texture_canvas.set_width(total_width);

        let total_height = rows * max_height;

        texture_canvas.set_height(total_height);
        let texture_ctx = texture_canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();

        // Make sure that everything is red before drawing the text
        // The red color will indicate empty space
        texture_ctx.set_fill_style(&JsValue::from_str("rgb(255,0,0)"));
        texture_ctx.fill_rect(0.0, 0.0, total_width as f64, total_height as f64);

        // Now prepare for drawing the text
        texture_ctx.set_line_width(line_width * font_size as f64);
        texture_ctx.set_font(font_string);

        // Due to lack of proper text metrics, we will have to do this dirty approximation
        let mut draw_y = (max_height * 4 / 5) as f64;

        let mut min_y = 0;
        let mut draw_x = 0;

        let mut chars_in_this_row = 0;

        let mut character_map = vec![None; max_char_code + 1];

        let mut index = 0;
        for character in chars.chars() {

            let mut substring = [0; 4];
            let min_x = draw_x;

            // The green color will indicate the interior of the text
            texture_ctx.set_fill_style(&JsValue::from_str("rgb(0,255,0)"));
            texture_ctx.fill_text(character.encode_utf8(&mut substring), draw_x as f64, draw_y).unwrap();

            // The blue color will indicate the border of the text
            texture_ctx.set_stroke_style(&JsValue::from_str("rgb(0,0,255)"));
            texture_ctx.stroke_text(character.encode_utf8(&mut substring), draw_x as f64, draw_y).unwrap();

            draw_x += char_sizes[index].0 + 2 * line_margin;

            let max_x = draw_x - line_margin;
            let max_y = min_y + max_height - 1;

            character_map[character as usize] = Some(Character::new(total_width, total_height, min_x, min_y, max_x, max_y));

            chars_in_this_row += 1;
            if chars_in_this_row >= chars_per_row {
                chars_in_this_row = 0;
                draw_x = 0;
                draw_y += max_height as f64;
                min_y += max_height;
            }

            index += 1;
        }

        // Temporarily for testing purposes:
        document.body().unwrap().append_child(&texture_canvas).unwrap();

        // Now we have drawn all text onto the canvas, so it's time to convert it to a WebGL texture
        let image_data = texture_ctx.get_image_data(0.0, 0.0, total_width as f64, total_height as f64).unwrap();

        let texture = gl.create_texture().unwrap();
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
        gl.tex_image_2d_with_u32_and_u32_and_image_data(GL::TEXTURE_2D, 0, GL::RGBA as i32, 
            GL::RGBA, GL::UNSIGNED_BYTE, &image_data).unwrap();
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

        Font {
            font_details,
            max_text_height: max_height,

            // The initial aspect_ratio doesn't matter because the TextRenderer will update the aspect_ratio of this font before every frame
            aspect_ratio: Cell::new(1.0),

            id: font_id,
            selected_font,

            characters: character_map,

            gl,
            shader_program,
            texture
        }
    }

    /// Gets the FontDetails instance that was used to create this Font. See the description of FontDetails for more info
    /// about such structs.
    pub fn get_font_details(&self) -> &FontDetails {
        &self.font_details
    }

    /// Creates a TextModel for the given string. The returned TextModel has a render method that will draw this text and can 
    /// be reused as often as you like. Reusing the returned TextModel is encouraged to avoid needless allocation of buffers.
    pub fn create_text_model(self: Rc<Self>, text: &str) -> TextModel {

        let mut char_counter = 0;
        for _char in text.chars() {
            char_counter += 1;
        }

        let gl = &self.gl;

        let buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

        let mut pos_x = 0;

        let position_floats_per_char = 12;
        let texture_floats_per_char = 12;

        let pos_factor_x = 1.0 / self.max_text_height as f32;
        let pos_max_y = 1.0;

        let mut buffer_data = vec![0.0; (position_floats_per_char + texture_floats_per_char) * char_counter];
        let mut char_index = 0;
        for text_char in text.chars() {

            let maybe_texture_char = self.characters[text_char as usize];
            
            match maybe_texture_char {
                Some(texture_char) => {
                    let offset = char_index * position_floats_per_char;

                    let min_x = pos_x as f32 * pos_factor_x;
                    let min_y = 0.0;
                    pos_x += texture_char.get_width();
                    let max_x = pos_x as f32 * pos_factor_x;
                    let max_y = pos_max_y;

                    buffer_data[offset + 0] = min_x;
                    buffer_data[offset + 1] = min_y;

                    buffer_data[offset + 2] = max_x;
                    buffer_data[offset + 3] = min_y;

                    buffer_data[offset + 4] = max_x;
                    buffer_data[offset + 5] = max_y;

                    buffer_data[offset + 6] = max_x;
                    buffer_data[offset + 7] = max_y;

                    buffer_data[offset + 8] = min_x;
                    buffer_data[offset + 9] = max_y;

                    buffer_data[offset + 10] = min_x;
                    buffer_data[offset + 11] = min_y;
                }, None => print(&format!("No texture for character {}", text_char))
            };

            char_index += 1;
        }

        let max_width = pos_x as f32 * pos_factor_x;

        let mut char_index = 0;
        for text_char in text.chars() {
            let maybe_texture_char = self.characters[text_char as usize];
            
            match maybe_texture_char {
                Some(texture_char) => {
                    let left_u = texture_char.get_left_u();
                    let bottom_v = texture_char.get_bottom_v();
                    let right_u = texture_char.get_right_u();
                    let top_v = texture_char.get_top_v();
                    let offset = position_floats_per_char * char_counter + char_index * texture_floats_per_char;

                    buffer_data[offset + 0] = left_u;
                    buffer_data[offset + 1] = bottom_v;

                    buffer_data[offset + 2] = right_u;
                    buffer_data[offset + 3] = bottom_v;

                    buffer_data[offset + 4] = right_u;
                    buffer_data[offset + 5] = top_v;

                    buffer_data[offset + 6] = right_u;
                    buffer_data[offset + 7] = top_v;

                    buffer_data[offset + 8] = left_u;
                    buffer_data[offset + 9] = top_v;

                    buffer_data[offset + 10] = left_u;
                    buffer_data[offset + 11] = bottom_v;
                }, None => print(&format!("No texture for character {}", text_char))
            };

            char_index += 1;
        }

        // Really? Is there no safe way to do this?
        unsafe {
            let js_array = Float32Array::view(&buffer_data);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_array, GL::STATIC_DRAW);
        }

        TextModel::new(Rc::clone(&self), buffer, char_counter, max_width)
    }

    pub(super) fn set_current(&self){
        self.gl.active_texture(GL::TEXTURE0);
        self.gl.bind_texture(GL::TEXTURE_2D, Some(&self.texture));
        let shader = self.shader_program.borrow();
        shader.set_texture_sampler(0);
    }

    pub(super) fn set_aspect_ratio(&self, aspect_ratio: f32){
        self.aspect_ratio.set(aspect_ratio);
    }
}

impl Drop for Font {

    fn drop(&mut self){
        self.gl.delete_texture(Some(&self.texture));
    }
}