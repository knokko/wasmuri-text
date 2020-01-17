use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;
use web_sys::HtmlCanvasElement;

use wasm_bindgen::JsCast;

use std::rc::Rc;
use std::cell::{
    Cell,
    RefCell
};

mod character;
mod shaders;
mod model;
mod font;

pub use model::*;
pub use font::*;

use shaders::TextProgram;

/// The TextRenderer is the main struct of this crate. Instances of TextRenderer can create Font's, which can create TextModel's
/// to perform the actual text rendering. You will need an instance of TextRenderer for each canvas you wish to draw text on with 
/// WebGl, so you will only need a single instance in most cases.
/// 
/// To get started, you will need to obtain the WebGlRenderingContext you wish to draw text on. Then you will need to create an
/// instance of TextRenderer. You can create one with the from_... functions of this struct. Use the function that is the most 
/// convenient for your situation and note that a WebGlRenderingContext can easily be cloned.
/// 
/// Once you have the instance, you need to add Font's. You can create a single Font at a time using the add_font method or you
/// can add multiple using the add_fonts method. If you use the add_font method, a reference to the newly created font will be
/// returned. If you use add_fonts, you can get the reference to the Font you want by using the get_font_by_details method.
/// 
/// Once you have a reference to the Font you wish to use, you can create a model for the text you want to draw. You will need a 
/// separate TextModel for each string you would like to draw. To create a TextModel, use the create_text_model method of the Font.
/// 
/// Before you start drawing the TextModel, call the start_rendering method of the TextRenderer. Thereafter, you can use the render
/// method of the TextModel to finally draw the text.
/// 
/// Every method mentioned above has its own more detailed description.
pub struct TextRenderer {

    gl: Rc<WebGlRenderingContext>,
    fonts: Vec<Rc<Font>>,

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
    pub all_chars: String,

    selected_font: Rc<Cell<Option<FontID>>>,

    shader_program: Rc<RefCell<TextProgram>>
}

pub const DEFAULT_FONT_SIZE: usize = 250;
pub const DEFAULT_LINE_WIDTH: f64 = 0.02;
pub const DEFAULT_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZáçéíóúýÁÇÉÍÓÚÝ 0123456789!@#$%^&*?<>:\"';[]{}()|\\/.,-_=+€`~";

impl TextRenderer {

    /// This function will create a TextRenderer instance from the given reference counter. This function is convenient if
    /// you are already using an Rc to store your webgl context. The created TextRenderer won't have any fonts yet, read the
    /// description of TextRenderer for more information about this.
    pub fn from_rc(gl: Rc<WebGlRenderingContext>) -> TextRenderer {
        let shader_program = Rc::new(RefCell::new(TextProgram::create_instance(Rc::clone(&gl))));
        let fonts = Vec::new();

        TextRenderer {
            gl,
            fonts,

            font_size: DEFAULT_FONT_SIZE,
            line_width: DEFAULT_LINE_WIDTH,
            all_chars: DEFAULT_CHARS.to_string(),

            selected_font: Rc::new(Cell::new(None)),
            shader_program
        }
    }

    /// This function will create a TextRenderer instance for the given webgl rendering context. The created TextRenderer 
    /// won't have any fonts yet, read the description of TextRenderer for more information about this.
    pub fn from_gl(gl: WebGlRenderingContext) -> TextRenderer {
        TextRenderer::from_rc(Rc::new(gl))
    }

    /// This function will create a TextRenderer instance for the given canvas. This method will panic if no webgl context
    /// can be created for the canvas. Even though browsers that support WebAssembly generally support WebGl, only 1 type of
    /// context can be created for each canvas, so this would fail if the canvas has created a 2d context before.
    pub fn from_canvas(canvas: &HtmlCanvasElement) -> TextRenderer {
        TextRenderer::from_gl(canvas.get_context("webgl").expect("should have get_context").expect("should be able to obtain webgl context").dyn_into::<WebGlRenderingContext>().expect("getContext'webgl' should give a webgl rendering context"))
    }

    /// Adds a Font for every FontDetails supplied to this method. After this method call, you can use the get_font_by_details
    /// method to obtain references to the created Font's.
    /// 
    /// Please note that creating a Font is an expensive operation, so you should not create more Font's than you need and reuse
    /// Font's rather than creating a new one every time you render text.
    /// 
    /// This method will use the current font_size, line_width and all_chars values of this TextRenderer and all created Font's
    /// will keep those values even if the values of this TextRenderer would be changed after this call. For more information
    /// about any of the three properties, see their description.
    pub fn add_fonts(&mut self, fonts: Vec<FontDetails>){
        let mut new_fonts = Vec::with_capacity(fonts.len());
        for font_details in fonts {
            new_fonts.push(Self::create_font(&self.gl, &self.shader_program, FontID::new(self.fonts.len()), &self.selected_font, self.font_size, self.line_width, font_details, &self.all_chars));
        }
        self.fonts.append(&mut new_fonts);
    }

    /// Adds a single Font with the given FontDetails. A reference to the newly created Font will be returned by this method. You
    /// could also retrieve the created Font with the get_font_by_details method of this TextRenderer.
    /// 
    /// Please note that creating a Font is an expensive operation, so you should not create more Font's than you need and reuse
    /// Font's rather than creating a new one every time you render text.
    /// 
    /// This method will use the current font_size, line_width and all_chars values of this TextRenderer and the created Font
    /// will keep those values even if the values of this TextRenderer would be changed after this call. For more information
    /// about any of the three properties, see their description.
    pub fn add_font(&mut self, font_details: FontDetails) -> Rc<Font> {
        let font = Self::create_font(&self.gl, &self.shader_program, FontID::new(self.fonts.len()), &self.selected_font, self.font_size, self.line_width, font_details, &self.all_chars);
        self.fonts.push(font);
        Rc::clone(&self.fonts[self.fonts.len() - 1])
    }

    fn create_font(gl: &Rc<WebGlRenderingContext>, shader_program: &Rc<RefCell<TextProgram>>, font_id: FontID, selected_font: &Rc<Cell<Option<FontID>>>, font_size: usize, line_width: f64, font_details: FontDetails, all_chars: &str) -> Rc<Font> {
        Rc::new(Font::new(Rc::clone(gl), Rc::clone(shader_program), font_id, Rc::clone(selected_font), font_size, line_width, font_details, all_chars))
    }

    /// Gets a previously created Font (with add_font or add_fonts) by its FontDetails. It will return the reference to the first
    /// Font with the same FontDetails as font_details, or None if no such Font was found. The font details will be compared by
    /// value, not by reference, so the supplied font_details does not need to have the same memory address as the original one of
    /// the Font.
    pub fn get_font_by_details(&self, font_details: FontDetails) -> Option<Rc<Font>> {

        // Don't bother doing clever search because I am expecting the number of fonts to be small
        for font in &self.fonts {
            if *font.get_font_details() == font_details {
                return Some(Rc::clone(font));
            }
        }

        None
    }

    /// This method should be called before doing any rendering operations with the Font's of this TextManager (it will do stuff like
    /// preparing the text shaders). This method will need to be called again if any external webgl rendering on the webgl context of this
    /// TextRenderer has taken place. With external, I mean any rendering that wasn't done by this crate.
    pub fn start_rendering(&mut self){

        let gl = &self.gl;
        let maybe_bound_canvas = gl.canvas();

        // We don't know what happened before the GUI rendering, so let's not make any assumptions about our current font
        self.selected_font.set(None);

        // If there is no canvas bound to it anymore, don't bother rendering
        if maybe_bound_canvas.is_some() {

            // The fonts need to know the aspect ratio for nice text rendering
            let bound_canvas = maybe_bound_canvas.unwrap().dyn_into::<HtmlCanvasElement>().expect("The bound webgl canvas should be a canvas element");
            let aspect_ratio = bound_canvas.width() as f32 / bound_canvas.height() as f32;
            for font in &self.fonts {
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