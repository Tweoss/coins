use log::Level;
use mogwai::prelude::*;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::console::log;
use web_sys::{Request, RequestInit, RequestMode, Response};

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

struct Login {
	username: String,
	count: i32,
	coins: Vec<Gizmo<coin::Coin>>,
}

#[derive(Clone)]
enum LoginIn {
	/// From the coin models, result of a coin flip
	Flipped(bool),
	/// From main, triggers a request to check current count
	Load,
	/// Loaded. Containes the count, newline, id
	Loaded(String),
	None,
}

#[derive(Clone)]
enum LoginOut {
	/// Send to the view the new count
	Count(i32),
	/// Send to the view the new count and the new id
	LoadedId(String),
}

impl Component for Login {
	type ModelMsg = LoginIn;
	type ViewMsg = LoginOut;
	type DomNode = HtmlElement;

	fn update(
		&mut self,
		msg: &LoginIn,
		tx_view: &Transmitter<LoginOut>,
		subscriber: &Subscriber<LoginIn>,
	) {
		match msg {
			LoginIn::Flipped(flipped) => {
				if *flipped {
					self.count += 1;
				} else {
					self.count -= 1;
				}
				tx_view.send(&LoginOut::Count(self.count));
			}
			LoginIn::Load => {
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
						LoginIn::Loaded(
							JsFuture::from(response.text().unwrap())
								.await
								.unwrap()
								.as_string()
								.unwrap(),
						)
					} else {
						LoginIn::None
					}
				});
				subscriber.subscribe(&rx);
			}
			LoginIn::Loaded(string) => {
				let split: Vec<&str> = string.splitn(2, '\n').collect();
				self.count = split[0].parse::<i32>().unwrap();
				self.username = split[1].to_string();
				tx_view.send(&LoginOut::LoadedId(self.username.clone()));
				tx_view.send(&LoginOut::Count(self.count));
			}
			_ => {}
		}
	}

	#[allow(unused_braces)]
	fn view(&self, tx: &Transmitter<LoginIn>, rx: &Receiver<LoginOut>) -> ViewBuilder<HtmlElement> {
		let rx_count = rx.branch_filter_map(|msg: &LoginOut| match msg {
			LoginOut::Count(count) => Some(format!("{}", count)),
			_ => None,
		});
		
		self.coins.iter().for_each(|g| {
			g.recv.clone().forward_filter_map(tx, |m: &coin::CoinOut| {
				let coin::CoinOut::Flipped(flipped) = m;
				Some(LoginIn::Flipped(*flipped))
			});
		});

		let rx_load = rx.branch_filter_map(|msg: &LoginOut| match msg {
			LoginOut::LoadedId(string) => Some(string.to_string() + " cents"),
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
		</div>
		)
	}
}

#[wasm_bindgen]
pub fn main(parent_id: Option<String>) -> Result<(), JsValue> {
	panic::set_hook(Box::new(console_error_panic_hook::hook));
	console_log::init_with_level(Level::Trace).unwrap();

	let gizmo = Gizmo::from(Login {
		username: "".to_string(),
		count: 0,
		coins: (0..3)
			.map(|i| coin::Coin { arm: i })
			.map(Gizmo::from)
			.collect(),
	});
	let view = View::from(gizmo.view_builder());
	gizmo.send(&LoginIn::Load);

	if let Some(id) = parent_id {
		let parent = utils::document().get_element_by_id(&id).unwrap();
		view.run_in_container(&parent)
	} else {
		view.run()
	}
}
