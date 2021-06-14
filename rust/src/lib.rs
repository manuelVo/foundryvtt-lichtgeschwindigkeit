mod geometry;
mod ptr_indexed_hash_set;
mod raycasting;
mod serialization;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
