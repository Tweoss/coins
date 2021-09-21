use crate::AppState;
use actix::Addr;
use actix_files::NamedFile;
use actix_web::{
	cookie, get, http, post, web, HttpMessage, HttpRequest, HttpResponse, Responder, Result,
};
use rand::{distributions::Bernoulli, rngs::ThreadRng};
use rand_distr::Distribution;
use std::path::PathBuf;
use uuid::Uuid;

pub struct ApplicationState {
	pub rng: ThreadRng,
	pub addr: Addr<AppState>,
	probabilities: Vec<Bernoulli>,
}

impl ApplicationState {
	pub fn new(rng: ThreadRng, addr: Addr<AppState>, probabilities: Vec<f64>) -> Self {
		ApplicationState {
			rng,
			addr,
			probabilities: probabilities
				.iter()
				.map(|p| Bernoulli::new(*p).unwrap())
				.collect::<Vec<Bernoulli>>(),
		}
	}
	pub fn get_bernoulli(&self, i: usize) -> Bernoulli {
		*self
			.probabilities
			.get(i)
			.unwrap_or(&Bernoulli::new(0.0).unwrap())
	}
}

/// Set a cookie for 2 hours involving a uuid and the chosen name.
/// Will overwrite any existing cookie.
/// No redirecting
#[get("/cookie/{id}")]
pub async fn set_cookie(path: web::Path<String>) -> HttpResponse {
	let cookie = (cookie::Cookie::build("id", Uuid::new_v4().to_string() + "_" + path.as_str()))
		.max_age(time::Duration::hours(2))
		.path("/")
		.same_site(cookie::SameSite::Strict)
		.finish();
	let rep = HttpResponse::build(http::StatusCode::OK)
		.cookie(cookie)
		.content_type("plain/text")
		.body("Set Cookie");
	rep
}

/// Redirects to the game page.
#[get("/redirect")]
pub async fn redirect() -> HttpResponse {
	HttpResponse::build(http::StatusCode::FOUND)
		.header(http::header::LOCATION, "/game/".to_string())
		.finish()
}

/// Handling for game page
pub async fn game_html() -> Result<NamedFile> {
	Ok(NamedFile::open("../game/index.html")?)
}

/// Handling for static game path files
#[get("/game/pkg/{filename}.{ext}")]
pub async fn game_files(path: web::Path<(String, String)>) -> Result<NamedFile> {
	let (filename, ext) = path.into_inner();
	Ok(NamedFile::open(
		("../game/pkg/".to_string() + &filename + "." + &ext)
			.parse::<PathBuf>()
			.unwrap(),
	)?)
}

/// Handling for game styling
#[get("/game/style/styles.css")]
pub async fn game_style() -> Result<NamedFile> {
	Ok(NamedFile::open(
		"../game/style/styles.css"
			.to_string()
			.parse::<PathBuf>()
			.unwrap(),
	)?)
}

#[get("/")]
pub async fn index() -> Result<NamedFile> {
	Ok(NamedFile::open("../login/index.html")?)
}

/// Handling for game styling
#[get("/pkg/{filename}.{ext}")]
pub async fn index_files(path: web::Path<(String, String)>) -> Result<NamedFile> {
	let (filename, ext) = path.into_inner();
	Ok(NamedFile::open(
		("../login/pkg/".to_string() + &filename + "." + &ext)
			.parse::<PathBuf>()
			.unwrap(),
	)?)
}
/// Handling for game styling
#[get("/style/styles.css")]
pub async fn index_style() -> Result<NamedFile> {
	Ok(NamedFile::open(
		"../login/style/styles.css"
			.to_string()
			.parse::<PathBuf>()
			.unwrap(),
	)?)
}

/// Flip a coin using the thread rng handle. Send the result to application
#[get("/flip/{coin}")]
pub async fn flip(req: HttpRequest) -> impl Responder {
	use crate::app::CoinFlipped;

	let coin = req
		.match_info()
		.get("coin")
		.map(|s| s.parse::<usize>().ok())
		.flatten()
		.unwrap_or(0); // if invalid, number defaults to first coin
	let app_data = req.app_data::<web::Data<ApplicationState>>().unwrap();
	let mut rng = app_data.rng.clone();
	let addr = &app_data.addr;
	let result: bool = app_data.get_bernoulli(coin).sample(&mut rng);
	addr.do_send(CoinFlipped {
		user_id: req.cookie("id").unwrap().value().to_string(),
		arm: coin,
		result,
	});
	HttpResponse::build(http::StatusCode::OK)
		.content_type("plain/text")
		.body(format!("{}", result))
}

#[get("/count")]
pub async fn count(req: HttpRequest) -> impl Responder {
	use crate::app::GetCount;
	let id = req.cookie("id").unwrap().value().to_string();
	let app_data = req.app_data::<web::Data<ApplicationState>>().unwrap();
	let addr = &app_data.addr;
	let count = addr
		.send(GetCount { id: id.clone() })
		.await
		.expect("Failed to get count");
	HttpResponse::build(http::StatusCode::OK)
		.content_type("plain/text")
		.body(format!("{}\n{}", count, id))
}

/// Send a message to the application to flush state
#[post("/flush")]
pub async fn flush(req: HttpRequest) -> impl Responder {
	use crate::app::Flush;
	let app_data = req.app_data::<web::Data<ApplicationState>>().unwrap();
	let addr = &app_data.addr;
	addr.do_send(Flush {});
	HttpResponse::Ok()
		.content_type("plain/text")
		.body("Sent Application message to flush")
}

pub async fn not_found() -> HttpResponse {
	HttpResponse::build(http::StatusCode::FOUND)
		.header(http::header::LOCATION, "/".to_string())
		.finish()
}
