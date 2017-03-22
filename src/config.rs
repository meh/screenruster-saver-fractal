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

use std::str::FromStr;
use screen::json::JsonValue;
use palette::{Gradient, Rgb};
use regex::Regex;
use meval::Expr;

pub struct Config {
	pub gradient:    Gradient<Rgb>,
	pub definitions: Vec<Definition>,
}

impl Default for Config {
	fn default() -> Config {
		Config {
			gradient: Gradient::new(vec![
				Rgb::new(0.0, 0.0, 0.0),
				Rgb::new(1.0, 1.0, 1.0),
				Rgb::new(0.0, 0.0, 0.0),
			]),

			definitions: Vec::new(),
		}
	}
}

#[derive(Default)]
pub struct Definition {
	pub gradient:  Option<Gradient<Rgb>>,
	pub algorithm: Algorithm,
}

pub enum Algorithm {
	None,
	Mandelbrot(Mandelbrot),
	Julia(Julia),
}

impl Default for Algorithm {
	fn default() -> Algorithm {
		Algorithm::None
	}
}

pub struct Mandelbrot {
	pub t:     Box<Fn(f64) -> f64>,
	pub iter:  Box<Fn(f64) -> f64>,
	pub scale: Box<Fn(f64) -> f64>,

	pub x: Box<Fn(f64) -> f64>,
	pub y: Box<Fn(f64) -> f64>,
}

impl Default for Mandelbrot {
	fn default() -> Mandelbrot {
		Mandelbrot {
			t:     box |_| 0.0,
			iter:  box |_| 70.0,
			scale: box |_| 2.5,

			x: box |_| 0.7,
			y: box |_| 0.0,
		}
	}
}

impl Mandelbrot {
	pub fn t(&self, tick: u64) -> f32 {
		(self.t)(tick as f64) as f32
	}

	pub fn iter(&self, t: f32) -> i32 {
		(self.iter)(t as f64) as i32
	}

	pub fn scale(&self, t: f32) -> f32 {
		(self.scale)(t as f64) as f32
	}

	pub fn x(&self, t: f32) -> f32 {
		(self.x)(t as f64) as f32
	}

	pub fn y(&self, t: f32) -> f32 {
		(self.y)(t as f64) as f32
	}
}

pub struct Julia {
	pub t:    Box<Fn(f64) -> f64>,
	pub iter: Box<Fn(f64) -> f64>,

	pub r: Box<Fn(f64) -> f64>,
	pub i: Box<Fn(f64) -> f64>,
}

impl Default for Julia {
	fn default() -> Julia {
		Julia {
			iter: Expr::from_str("60").unwrap().bind("t").unwrap(),

			t: Expr::from_str("tick").unwrap().bind("tick").unwrap(),
			r: Expr::from_str("t").unwrap().bind("t").unwrap(),
			i: Expr::from_str("t").unwrap().bind("t").unwrap(),
		}
	}
}

impl Julia {
	pub fn t(&self, tick: u64) -> f32 {
		(self.t)(tick as f64) as f32
	}

	pub fn iter(&self, t: f32) -> i32 {
		(self.iter)(t as f64) as i32
	}

	pub fn r(&self, t: f32) -> f32 {
		(self.r)(t as f64) as f32
	}

	pub fn i(&self, t: f32) -> f32 {
		(self.i)(t as f64) as f32
	}
}

impl Config {
	pub fn new(table: JsonValue) -> Config {
		let mut config = Config::default();

		for table in table["define"].members() {
			let mut definition = Definition::default();

			if table["gradient"].is_array() {
				let mut colors = Vec::new();
				let     regex  = Regex::new(r"#([:xdigit:]{2})([:xdigit:]{2})([:xdigit:]{2})").unwrap();

				for color in table["gradient"].members() {
					if let Some(string) = color.as_str() {
						if let Some(captures) = regex.captures(string) {
							colors.push(Rgb::new_u8(
								u8::from_str_radix(captures.get(1).map(|c| c.as_str()).unwrap_or("0"), 16).unwrap_or(0),
								u8::from_str_radix(captures.get(2).map(|c| c.as_str()).unwrap_or("0"), 16).unwrap_or(0),
								u8::from_str_radix(captures.get(3).map(|c| c.as_str()).unwrap_or("0"), 16).unwrap_or(0),
							));
						}
					}
				}

				definition.gradient = Some(Gradient::new(colors));
			}

			definition.algorithm = match table["algorithm"].as_str() {
				Some("mandelbrot") => {
					let mut algorithm = Mandelbrot::default();

					if let Some(t) = table["t"].as_str() {
						algorithm.t = Expr::from_str(t).unwrap().bind("tick").unwrap();
					}

					match table["iter"] {
						JsonValue::String(ref string) =>
							algorithm.iter = Expr::from_str(string).unwrap().bind("t").unwrap(),

						JsonValue::Number(number) =>
							algorithm.iter = box move |_| number.into(),

						_ => ()
					}

					match table["scale"] {
						JsonValue::String(ref string) =>
							algorithm.scale = Expr::from_str(string).unwrap().bind("t").unwrap(),

						JsonValue::Number(number) =>
							algorithm.scale = box move |_| number.into(),

						_ => ()
					}

					if let Some(x) = table["x"].as_str() {
						algorithm.x = Expr::from_str(x).unwrap().bind("t").unwrap();
					}

					if let Some(y) = table["y"].as_str() {
						algorithm.y = Expr::from_str(y).unwrap().bind("t").unwrap();
					}

					Algorithm::Mandelbrot(algorithm)
				}

				Some("julia") => {
					let mut algorithm = Julia::default();

					if let Some(t) = table["t"].as_str() {
						algorithm.t = Expr::from_str(t).unwrap().bind("tick").unwrap();
					}

					match table["iter"] {
						JsonValue::String(ref string) =>
							algorithm.iter = Expr::from_str(string).unwrap().bind("t").unwrap(),

						JsonValue::Number(number) =>
							algorithm.iter = box move |_| number.into(),

						_ => ()
					}

					if let Some(r) = table["r"].as_str() {
						algorithm.r = Expr::from_str(r).unwrap().bind("t").unwrap();
					}

					if let Some(i) = table["i"].as_str() {
						algorithm.i = Expr::from_str(i).unwrap().bind("t").unwrap();
					}

					Algorithm::Julia(algorithm)
				}

				_ =>
					Algorithm::default()
			};

			config.definitions.push(definition);
		}

		config
	}
}
