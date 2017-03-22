// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// This file is part of screenruster.
//
// screenruster is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// screenruster is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with screenruster.  If not, see <http://www.gnu.org/licenses/>.

use std::rc::Rc;

use rand::{self, Rng};

use screen;
use screen::json::JsonValue;
use screen::gl::{self, Surface};
use screen::image;

use config::{Config, Algorithm, Definition};
use {Vertex};

#[derive(Default)]
pub struct Saver {
	config: Option<Config>,
	state:  screen::State,
	gl:     Option<Graphics>,

	definition: Option<Definition>,
	tick:       u64,
}

unsafe impl Send for Saver { }

struct Graphics {
	width:  u32,
	height: u32,

	vertex:  gl::VertexBuffer<Vertex>,
	index:   gl::IndexBuffer<u16>,
	texture: gl::texture::Texture1d,

	mandelbrot: gl::Program,
	julia:      gl::Program,
}

impl screen::Saver for Saver {
	fn config(&mut self, config: JsonValue) {
		let mut config = Config::new(config);
		let     count  = config.definitions.len();

		self.definition = if !config.definitions.is_empty() {
			Some(config.definitions.remove(rand::thread_rng().gen_range(0, count)))
		}
		else {
			None
		};

		self.config = Some(config);
	}

	fn initialize(&mut self, context: Rc<gl::backend::Context>) {
		let config          = self.config.as_ref().unwrap();
		let (width, height) = context.get_framebuffer_dimensions();

		let vertex = gl::VertexBuffer::new(&context, &[
			Vertex { position: [-1.0, -1.0], texture: [0.0, 0.0] },
			Vertex { position: [-1.0,  1.0], texture: [0.0, 1.0] },
			Vertex { position: [ 1.0,  1.0], texture: [1.0, 1.0] },
			Vertex { position: [ 1.0, -1.0], texture: [1.0, 0.0] },
		]).unwrap();

		let index = gl::IndexBuffer::new(&context, gl::index::PrimitiveType::TriangleStrip,
			&[1 as u16, 2, 0, 3]).unwrap();

		let texture = {
			let mut image    = image::DynamicImage::new_rgb8(256, 1);
			let     gradient = if let Some(def) = self.definition.as_ref() {
				def.gradient.clone().unwrap_or(config.gradient.clone())
			}
			else {
				config.gradient.clone()
			};

			for (color, pixel) in gradient.take(256).zip(image.as_mut_rgb8().unwrap().pixels_mut()) {
				pixel.data = color.to_pixel();
			}

			let image = gl::texture::RawImage1d {
				data: image.to_rgb().into_raw().into(),
				width: 256,
				format: gl::texture::ClientFormat::U8U8U8,
			};

			gl::texture::Texture1d::with_mipmaps(&context, image, gl::texture::MipmapsOption::NoMipmap).unwrap()
		};

		macro_rules! load {
			($path:expr) => (
				program!(&context,
					110 => {
						vertex:   include_str!("../assets/shaders/vertex.glsl"),
						fragment: include_str!(concat!("../assets/shaders/", $path, ".glsl")),
					}
				).unwrap()
			);
		}

		let mandelbrot = load!("mandelbrot");
		let julia      = load!("julia");

		self.gl = Some(Graphics {
			width:   width,
			height:  height,

			vertex:  vertex,
			index:   index,
			texture: texture,

			mandelbrot: mandelbrot,
			julia:      julia,
		});
	}

	fn resize(&mut self, context: Rc<gl::backend::Context>) {
		let gl              = self.gl.as_mut().unwrap();
		let (width, height) = context.get_framebuffer_dimensions();

		gl.width  = width;
		gl.height = height;
	}

	fn start(&mut self) {
		self.state = screen::State::Running;
	}

	fn stop(&mut self) {
		self.state = screen::State::None;
	}

	fn state(&self) -> screen::State {
		self.state
	}

	fn update(&mut self) {
		self.tick += 1;
	}

	fn render<S: Surface>(&self, target: &mut S, _screen: &gl::texture::Texture2d) {
		let gl = self.gl.as_ref().unwrap();

		if let Some(def) = self.definition.as_ref() {
			match def.algorithm {
				Algorithm::None => (),

				Algorithm::Mandelbrot(ref conf) => {
					let t     = conf.t(self.tick);
					let iter  = conf.iter(t);
					let scale = conf.scale(t);

					let x = conf.x(t);
					let y = conf.y(t);

					let uniforms = uniform! {
						center:     (x, y),
						scale:      scale,
						iterations: iter,

						colors: gl.texture.sampled()
							.minify_filter(gl::uniforms::MinifySamplerFilter::Nearest)
							.magnify_filter(gl::uniforms::MagnifySamplerFilter::Nearest),
					};

					target.draw(&gl.vertex, &gl.index, &gl.mandelbrot, &uniforms, &Default::default()).unwrap();
				}

				Algorithm::Julia(ref conf) => {
					let t    = conf.t(self.tick);
					let iter = conf.iter(t);
					let seed = (conf.r(t), conf.i(t));

					let uniforms = uniform! {
						iterations: iter,
						seed:       seed,

						colors: gl.texture.sampled()
							.minify_filter(gl::uniforms::MinifySamplerFilter::Nearest)
							.magnify_filter(gl::uniforms::MagnifySamplerFilter::Nearest),
					};

					target.draw(&gl.vertex, &gl.index, &gl.julia, &uniforms, &Default::default()).unwrap();
				}
			}
		}
	}
}
