use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;
use web_sys::HtmlCanvasElement;

use wasm_bindgen::JsCast;

use std::rc::Rc;
use std::cell::RefCell;

pub use model::*;
pub use font::*;

mod character;
mod shaders;

use shaders::TextProgram;

pub struct TextRenderer<'a> {

    gl: Rc<WebGlRenderingContext>,
    fonts: Vec<Font<'a>>,

    /// The font_size that will be used to draw the backing textures of the characters. Changing this value
    /// will affect only the fonts that were added after changing the value (with the add_font or add_fonts method).
    /// This allows users to give different font sizes to different fonts.
    /// 
    /// Note that the font size will not affect the size of the characters that the next fonts will draw onto the screen,
    /// it will only affect the level of detail of the font and the memory usage. A bigger font size will make the drawn
    /// characters more detailed, but this is usually only noticable when you are drawing the characters using a very big scale.
    /// 
    /// Usually, users would not need to modify this value because I believe the default value is quite reasonable, but they are
    /// free to do so if they disagree.
    pub font_size: usize,

    /// The line_width determines the width of the (stroking) lines surrounding drawn characters. The value is a fraction of the
    /// font_size, so a value of 0.03 with a font_size of 100 would result in a line width of 3 in the backing texture of the font.
    /// 
    /// The font will allow you to choose a fill_color and a stroke_color when drawing text. The lines surrounding the characters of
    /// the text will get the stroke_color.
    /// 
    /// Setting this value to 0 will prevent the surrounding lines from being drawn and will make sure that the stroke_color parameter
    /// of text drawing methods will be visible in the drawn text.
    /// 
    /// Usually, users would not need to modify this value because I believe the default value is quite reasonable, but they are
    /// free to do so if they disagree.
    pub line_width: f64,

    /// The all_chars is a string containing all characters that fonts will be able to draw. If you attempt to draw a character
    /// that is not in this string, the character will not be drawn. Whenever a font is added (using add_font or add_fonts), it will
    /// be able to draw all characters that are in the current value of this string. Modifying this string thereafter will not have
    /// any effect on the fonts created before.
    /// 
    /// The default value contains the characters in the alphabet (both uppercase and lowercase and some accents), the number digits 
    /// and all special characters I could find on my keyboard. If you need to draw characters not in this string, you will need to 
    /// modify it before adding fonts. It will usually not be necessary, but I might have missed some characters or you might need 
    /// for instance Chinese characters. Please note that more characters means more memory usage.
    pub all_chars: &'a str,

    selected_font: RefCell<Option<FontID>>,

    shader_program: RefCell<TextProgram>
}

pub const DEFAULT_FONT_SIZE: usize = 250;
pub const DEFAULT_LINE_WIDTH: f64 = 0.02;
pub const DEFAULT_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZáçéíóúýÁÇÉÍÓÚÝ 0123456789!@#$%^&*?<>:\"';[]{}()|\\/.,-_=+€`~";

impl<'a> TextRenderer<'a> {

    /// This will create a new TextRenderer without any fonts. To add fonts, use the add_fonts or add_font method!
    pub fn new_empty(gl: Rc<WebGlRenderingContext>, expected_number_of_fonts: usize) -> TextRenderer<'a> {
        let shader_program = RefCell::new(TextProgram::create_instance(Rc::clone(&gl)));
        let fonts = Vec::with_capacity(expected_number_of_fonts);

        TextRenderer {
            gl,
            fonts,

            font_size: DEFAULT_FONT_SIZE,
            line_width: DEFAULT_LINE_WIDTH,
            all_chars: DEFAULT_CHARS,

            selected_font: RefCell::new(None),
            shader_program
        }
    }

    pub fn add_fonts(&'a mut self, fonts: Vec<FontDetails<'a>>){
        let mut new_fonts = Vec::with_capacity(fonts.len());
        for font_details in fonts {
            new_fonts.push(Self::create_font(&self.gl, &self.shader_program, FontID::new(self.fonts.len()), &self.selected_font, self.font_size, self.line_width, font_details, self.all_chars));
        }
        self.fonts.append(&mut new_fonts);
    }

    pub fn add_font(&'a mut self, font_details: FontDetails<'a>) -> &'a Font {
        let font = Self::create_font(&self.gl, &self.shader_program, FontID::new(self.fonts.len()), &self.selected_font, self.font_size, self.line_width, font_details, self.all_chars);
        self.fonts.push(font);
        &self.fonts[self.fonts.len() - 1]
    }

    fn create_font(gl: &'a WebGlRenderingContext, shader_program: &'a RefCell<TextProgram>, font_id: FontID, selected_font: &'a RefCell<Option<FontID>>, font_size: usize, line_width: f64, font_details: FontDetails<'a>, all_chars: &str) -> Font<'a> {
        Font::new(&gl, &shader_program, font_id, selected_font, font_size, line_width, font_details, all_chars)
    }

    pub fn get_font_by_details(&self, font_details: FontDetails<'a>) -> Option<&Font> {

        // Don't bother doing clever search because I am expecting the number of fonts to be small
        for font in &self.fonts {
            if font.get_font_details() == font_details {
                return Some(font);
            }
        }

        None
    }

    pub fn get_font_by_id(&self, font_id: FontID) -> &Font {
        &self.fonts[font_id.get_value()]
    }

    /// This method should be called before doing any rendering operations with the fonts
    pub fn start_rendering(&mut self){

        let gl = &self.gl;
        let maybe_bound_canvas = gl.canvas();

        // We don't know what happened before the GUI rendering, so let's not make any assumptions about our current font
        let mut selected_font = self.selected_font.borrow_mut();
        *selected_font = None;

        // If there is no canvas bound to it anymore, don't bother rendering
        if maybe_bound_canvas.is_some() {

            // The fonts need to know the aspect ratio for nice text rendering
            let bound_canvas = maybe_bound_canvas.unwrap().dyn_into::<HtmlCanvasElement>().expect("The bound webgl canvas should be a canvas element");
            let aspect_ratio = bound_canvas.width() as f32 / bound_canvas.height() as f32;
            for font in &mut self.fonts {
                font.set_aspect_ratio(aspect_ratio);
            }

            // And finally actually start rendering
            let shader = self.shader_program.borrow();
            shader.use_program();
            gl.enable(GL::BLEND);
            gl.blend_func_separate(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA, GL::ONE, GL::ONE_MINUS_SRC_ALPHA);
        }
    }
}