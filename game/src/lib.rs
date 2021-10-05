use log::Level;
use mogwai::prelude::*;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::console::log;
use web_sys::{Request, RequestInit, RequestMode, Response};

mod audio;
mod coin;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (let mut __a__ = js_sys::Array::new(); __a__.set(0, format_args!($($t)*).to_string().into()); log(&__a__))
}

struct Game {
	username: String,
	count: i32,
	coins: Vec<Gizmo<coin::Coin>>,
	next_id: usize,
	audios: Vec<Gizmo<audio::Audio>>,
}

#[derive(Clone)]
enum GameIn {
	/// From the coin models, result of a coin flip
	Flipped(bool),
	/// From main, triggers a request to check current count
	Load,
	/// Loaded. Containes the count, newline, id
	Loaded(String),
	/// Create a new item
	NewAudio(bool),
	/// Remove the item at the given index
	RemoveAudio(usize),
	None,
}

#[derive(Clone)]
enum GameOut {
	/// Send to the view the new count
	Count(i32),
	/// Send to the view the new count and the new id
	LoadedId(String),
	/// Patch the view of audios
	PatchAudio(Patch<View<HtmlElement>>),
}

impl Component for Game {
	type ModelMsg = GameIn;
	type ViewMsg = GameOut;
	type DomNode = HtmlElement;

	fn update(
		&mut self,
		msg: &GameIn,
		tx_view: &Transmitter<GameOut>,
		subscriber: &Subscriber<GameIn>,
	) {
		match msg {
			GameIn::Flipped(flipped) => {
				if *flipped {
					self.count += 1;
				} else {
					self.count -= 1;
				}
				tx_view.send(&GameOut::Count(self.count));
			}
			GameIn::Load => {
				let mut opts = RequestInit::new();
				opts.method("GET");
				opts.mode(RequestMode::SameOrigin);
				let req = Request::new_with_str_and_init("/count", &opts)
					.expect("Failed to create request");
				let (tx, rx) = txrx();
				tx.send_async(async move {
					let window = web_sys::window().unwrap();
					let response = JsFuture::from(window.fetch_with_request(&req))
						.await
						.expect("Failed to send request")
						.dyn_into::<Response>()
						.expect("Malformed response");
					if response.status() == 200 {
						console_log!("Successfully set cookie, response: {:?}", response);
						GameIn::Loaded(
							JsFuture::from(response.text().unwrap())
								.await
								.unwrap()
								.as_string()
								.unwrap(),
						)
					} else {
						GameIn::None
					}
				});
				subscriber.subscribe(&rx);
			}
			GameIn::Loaded(string) => {
				let split: Vec<&str> = string.splitn(2, '\n').collect();
				self.count = split[0].parse::<i32>().unwrap();
				self.username = split[1].to_string();
				tx_view.send(&GameOut::LoadedId(self.username.clone()));
				tx_view.send(&GameOut::Count(self.count));
			}
			GameIn::NewAudio(yes) => {
				let item = audio::Audio { id: self.next_id , yes: *yes};
				self.next_id += 1;
				console_log!("Saying hi again");

				let gizmo = Gizmo::from(item);
				subscriber.subscribe_filter_map(&gizmo.recv, |child_msg: &audio::AudioOut| {
					match child_msg {
						audio::AudioOut::Remove(index) => Some(GameIn::RemoveAudio(*index)),
					}
				});

				let view: View<HtmlElement> = View::from(gizmo.view_builder());
				tx_view.send(&GameOut::PatchAudio(Patch::PushBack { value: view }));
				self.audios.push(gizmo);
			}
			GameIn::RemoveAudio(id) => {
				let mut may_index = None;
				'find_item_by_id: for (item, index) in self.audios.iter().zip(0..) {
					if &item.state_ref().id == id {
						may_index = Some(index);
						tx_view.send(&GameOut::PatchAudio(Patch::Remove { index }));
						break 'find_item_by_id;
					}
				}
				if let Some(index) = may_index {
					self.audios.remove(index);
				}
			}
			_ => {}
		}
	}

	#[allow(unused_braces)]
	fn view(&self, tx: &Transmitter<GameIn>, rx: &Receiver<GameOut>) -> ViewBuilder<HtmlElement> {
		let rx_count = rx.branch_filter_map(|msg: &GameOut| match msg {
			GameOut::Count(count) => Some(format!("{}", count)),
			_ => None,
		});
		self.coins.iter().for_each(|g| {
			g.recv.clone().forward_filter_map(tx, |m: &coin::CoinOut| {
				let coin::CoinOut::Flipped(flipped) = m;
				Some(GameIn::Flipped(*flipped))
			});
			g.recv.clone().forward_filter_map(tx, |m: &coin::CoinOut| {
				let coin::CoinOut::Flipped(flipped) = m;
				console_log!("Saying HI");
				Some(GameIn::NewAudio(*flipped))
			});
		});

		let rx_load = rx.branch_filter_map(|msg: &GameOut| match msg {
			GameOut::LoadedId(string) => Some(string.to_string() + " cents"),
			_ => None,
		});

		builder!(
		<div class="container">
			<h1 class="text-center" style="pointer-events: none;">{("0", rx_count)}</h1>
			<p class="text-center">{("0 cents", rx_load)}</p>
			<div class="row row-cols-1 row-cols-md-3 row-cols-lg-3 row-cols-xl-3 row-cols-xxl-3">
				<div class="col">
					{self.coins[0].view_builder()}
				</div>
				<div class="col">
					{self.coins[1].view_builder()}
				</div>
				<div class="col">
					{self.coins[2].view_builder()}
				</div>
			</div>
			// container for audios
			<div patch:children=rx.branch_filter_map(|m: &GameOut | if let GameOut::PatchAudio(patch) = m { Some(patch.clone())} else { None} )>
			</div>
		</div>
		)
	}
}

#[wasm_bindgen]
pub fn main(parent_id: Option<String>) -> Result<(), JsValue> {
	panic::set_hook(Box::new(console_error_panic_hook::hook));
	console_log::init_with_level(Level::Trace).unwrap();

	let gizmo = Gizmo::from(Game {
		username: "".to_string(),
		count: 0,
		coins: (0..3)
			.map(|i| coin::Coin { arm: i })
			.map(Gizmo::from)
			.collect(),
		next_id: 0,
		audios: Vec::new(),
	});
	let view = View::from(gizmo.view_builder());
	gizmo.send(&GameIn::Load);

	if let Some(id) = parent_id {
		let parent = utils::document().get_element_by_id(&id).unwrap();
		view.run_in_container(&parent)
	} else {
		view.run()
	}
}
