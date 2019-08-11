const VERTEX_SOURCE: &str = "

attribute vec2 relativePosition;
attribute vec2 textureCoords;

varying vec2 passTextureCoords;

uniform vec2 screenPosition;
uniform vec2 scale;

void main(){
    gl_Position = vec4(screenPosition.x + scale.x * relativePosition.x, screenPosition.y + scale.y * relativePosition.y, 0.0, 1.0);
    passTextureCoords = textureCoords;
}
";

const FRAGMENT_SOURCE: &str = "

precision mediump float;

varying vec2 passTextureCoords;

uniform sampler2D textureSampler;

uniform vec4 fillColor;
uniform vec4 strokeColor;
uniform vec4 backgroundColor;

void main(){
    vec4 texelColor = texture2D(textureSampler, passTextureCoords);
    gl_FragColor = backgroundColor * texelColor.r + fillColor * texelColor.g + strokeColor * texelColor.b;
}
";

use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;

use web_sys::WebGlProgram;
use web_sys::WebGlShader;
use web_sys::WebGlUniformLocation;

use std::rc::Rc;

use wasmuri_core::util::color::Color;

pub struct TextProgram {

    gl: Rc<WebGlRenderingContext>,

    program: WebGlProgram,
    vertex_shader: WebGlShader,
    fragment_shader: WebGlShader,

    attrib_relative_position: i32,
    attrib_texture_coords: i32,

    uniform_texture_sampler: WebGlUniformLocation,

    uniform_screen_position: WebGlUniformLocation,
    uniform_scale: WebGlUniformLocation,

    uniform_fill_color: WebGlUniformLocation,
    uniform_stroke_color: WebGlUniformLocation,
    uniform_background_color: WebGlUniformLocation,

    current_screen_position: (f32, f32),
    current_scale: (f32, f32),

    current_fill_color: Color,
    current_stroke_color: Color,
    current_background_color: Color
}

impl TextProgram {

    pub fn create_instance(gl: Rc<WebGlRenderingContext>) -> TextProgram {

        let vertex_shader = gl.create_shader(GL::VERTEX_SHADER).expect("Couldn't create vertex shader");
        gl.shader_source(&vertex_shader, VERTEX_SOURCE);
        gl.compile_shader(&vertex_shader);
        if !gl.get_shader_parameter(&vertex_shader, GL::COMPILE_STATUS).as_bool().expect("Compile status of vertex shader was not a bool") {
            panic!("Couldn't compile vertex text shader {}", gl.get_shader_info_log(&vertex_shader).expect("Couldn't get shader info log of vertex shader"));
        }

        let fragment_shader = gl.create_shader(GL::FRAGMENT_SHADER).expect("Couldn't create fragment shader");
        gl.shader_source(&fragment_shader, FRAGMENT_SOURCE);
        gl.compile_shader(&fragment_shader);
        if !gl.get_shader_parameter(&fragment_shader, GL::COMPILE_STATUS).as_bool().expect("Compile status of fragmetn shader was not a bool") {
            panic!("Couldn't compile fragment text shader {}", gl.get_shader_info_log(&fragment_shader).expect("Couldn't get shader info log of fragment shader"));
        }

        let program = gl.create_program().expect("Couldn't create shader program");

        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        if !gl.get_program_parameter(&program, GL::LINK_STATUS).as_bool().expect("Couldn't get link status of text shader program") {
            panic!("Couldn't link the text shader program {}", gl.get_program_info_log(&program).expect("Couldn't get program info log of text shader program"));
        }

        let attrib_relative_position = gl.get_attrib_location(&program, "relativePosition");
        let attrib_texture_coords = gl.get_attrib_location(&program, "textureCoords");

        let uniform_texture_sampler = gl.get_uniform_location(&program, "textureSampler").expect("Couldn't get textureSampler uniform location");

        let uniform_screen_position = gl.get_uniform_location(&program, "screenPosition").expect("Couldn't get screenPosition uniform location");
        let uniform_scale = gl.get_uniform_location(&program, "scale").expect("Couldn't get scale uniform location");

        let uniform_fill_color = gl.get_uniform_location(&program, "fillColor").expect("Couldn't get fillColor uniform location");
        let uniform_stroke_color = gl.get_uniform_location(&program, "strokeColor").expect("Couldn't get strokeColor uniform lcoation");
        let uniform_background_color = gl.get_uniform_location(&program, "backgroundColor").expect("Couldn't get backgroundColor uniform location");

        TextProgram {
            gl,

            program,
            vertex_shader,
            fragment_shader,

            attrib_relative_position,
            attrib_texture_coords,

            uniform_texture_sampler,

            uniform_screen_position,
            uniform_scale,

            uniform_fill_color,
            uniform_stroke_color,
            uniform_background_color,

            current_screen_position: (0.0, 0.0),
            current_scale: (0.0, 0.0),

            current_fill_color: Color::from_rgba(0, 0, 0, 0),
            current_stroke_color: Color::from_rgba(0, 0, 0, 0),
            current_background_color: Color::from_rgba(0, 0, 0, 0)
        }
    }

    pub fn use_program(&self){
        self.gl.use_program(Some(&self.program));
    }

    pub fn set_texture_sampler(&self, texture_unit: i32){
        self.gl.uniform1i(Some(&self.uniform_texture_sampler), texture_unit);
    }

    pub fn set_screen_position(&mut self, x: f32, y: f32){
        if self.current_screen_position != (x, y){
            self.gl.uniform2f(Some(&self.uniform_screen_position), x, y);
            self.current_screen_position = (x, y);
        }
    }

    pub fn set_scale(&mut self, x: f32, y: f32){
        if self.current_scale != (x, y){
            self.gl.uniform2f(Some(&self.uniform_scale), x, y);
            self.current_scale = (x, y);
        }
    }

    fn set_color(&self, uniform: &WebGlUniformLocation, color: Color){
        self.gl.uniform4f(Some(uniform), color.get_red_float(), color.get_green_float(), color.get_blue_float(), color.get_alpha_float());
    }

    pub fn set_background_color(&mut self, background: Color){
        if self.current_background_color != background {
            self.set_color(&self.uniform_background_color, background);
            self.current_background_color = background;
        }
    }

    pub fn set_fill_color(&mut self, fill: Color){
        if self.current_fill_color != fill {
            self.set_color(&self.uniform_fill_color, fill);
            self.current_fill_color = fill;
        }
    }

    pub fn set_stroke_color(&mut self, stroke: Color){
        if self.current_stroke_color != stroke {
            self.set_color(&self.uniform_stroke_color, stroke);
            self.current_stroke_color = stroke;
        }
    }

    pub fn get_relative_position(&self) -> i32 {
        self.attrib_relative_position
    }

    pub fn get_texture_coords(&self) -> i32 {
        self.attrib_texture_coords
    }
}

impl Drop for TextProgram {

    fn drop(&mut self){
        self.gl.delete_program(Some(&self.program));
        self.gl.delete_shader(Some(&self.vertex_shader));
        self.gl.delete_shader(Some(&self.fragment_shader));
    }
}