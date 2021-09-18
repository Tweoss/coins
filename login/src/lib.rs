use log::Level;
use mogwai::prelude::*;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console::log, HtmlInputElement, KeyboardEvent};
use web_sys::{Request, RequestInit, RequestMode, Response};

#[allow(unused_braces)]
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

pub struct Login {
	pub username: String,
}

#[derive(Clone)]
pub enum LoginIn {
	Update(String),
	Submit,
	GotCookie,
	None,
}

#[derive(Clone)]
pub enum LoginOut {}

impl Component for Login {
	type ModelMsg = LoginIn;
	type ViewMsg = LoginOut;
	type DomNode = HtmlElement;

	fn update(
		&mut self,
		msg: &LoginIn,
		_tx_view: &Transmitter<LoginOut>,
		subscriber: &Subscriber<LoginIn>,
	) {
		match msg {
			LoginIn::Update(new_name) => {
				self.username = new_name.to_string();
			}
			LoginIn::Submit => {
				use urlencoding::encode;
				console_log!("Submitting username: {}", self.username);
				let mut opts = RequestInit::new();
				opts.method("GET");
				opts.mode(RequestMode::SameOrigin);
				let url = format!("/cookie/{}", encode(&self.username));
				// create an async request. send, take response, and send it on the transmitter
				// subscribe to the receiver via subscriber, then act on the response
				let (tx, rx) = txrx();
				tx.send_async(async move {
					let window = web_sys::window().unwrap();
					let request =
						Request::new_with_str_and_init(&url, &opts).expect("Should be valid URL");
					let response = JsFuture::from(window.fetch_with_request(&request))
						.await
						.expect("Failed to send request")
						.dyn_into::<Response>()
						.expect("Malformed response");
					if response.status() == 200 {
						console_log!("Successfully set cookie, response: {:?}", response);
						LoginIn::GotCookie
					} else {
						LoginIn::None
					}
				});
				subscriber.subscribe(&rx);
			}
			LoginIn::GotCookie => {
				let window = web_sys::window().unwrap();
				let location = window.location();
				location.set_href("/redirect").expect("Unable to redirect");
			}
			LoginIn::None => {}
		}
	}

	// Notice that the `Component::view` function returns a `ViewBuilder<T>` and not
	// a `View<T>`.
	fn view(
		&self,
		tx: &Transmitter<LoginIn>,
		_rx: &Receiver<LoginOut>,
	) -> ViewBuilder<HtmlElement> {
		let tx_update = tx.contra_map(|e: &Event| {
			LoginIn::Update(
				e.target()
					.expect("Must have target for event")
					.unchecked_ref::<HtmlInputElement>()
					.value()
					.trim()
					.to_string(),
			)
		});
		let tx_submit = tx.contra_map(|_: &Event| LoginIn::Submit);
		let tx_enter = tx.contra_map(|e: &Event| {
			if e.unchecked_ref::<KeyboardEvent>().key() == "Enter" {
				LoginIn::Submit
			} else {
				LoginIn::None
			}
		});

		builder!(
			<div class="container d-md-flex justify-content-md-center align-items-md-center">
				<div class="d-md-flex justify-content-md-center align-items-md-center" style="width: 50vw;height: 50vw;position: absolute;top: 50%;left: 50%;margin-left: -25vw;margin-top: -25vw;" onsubmit="return false">
					<form class="d-md-flex justify-content-center">
						<input on:input=tx_update on:keydown=tx_enter class="form-control" type="text" placeholder="username" name="username" autofocus="" autocomplete="on" required="" style="border-right-color: #00000000;"></input>
						<button on:click=tx_submit class="btn btn-primary" type="button">"enter"</button>
					</form>
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
	});
	let view = View::from(gizmo.view_builder());

	if let Some(id) = parent_id {
		let parent = utils::document().get_element_by_id(&id).unwrap();
		view.run_in_container(&parent)
	} else {
		view.run()
	}
}
