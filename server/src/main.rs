use actix::Actor;
use actix_web::{middleware, web, App, HttpServer};
use clap::Clap;
use rand::rngs::ThreadRng;

mod app;
use app::AppState;

mod cli;
use cli::Opts;

mod handlers;
use handlers::ApplicationState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let opts: Opts = Opts::parse();
	if opts.verbose {
		std::env::set_var("RUST_LOG", "actix_web=debug");
		env_logger::init();
		println!("Starting server.");
	}
	let app_addr =
		AppState::new(ThreadRng::default(), opts.coin_probs.clone(), opts.verbose).start();
	HttpServer::new(move || {
		App::new()
			.wrap(middleware::Logger::default())
			.data(ApplicationState::new(
				ThreadRng::default(),
				app_addr.clone(),
				opts.coin_probs.clone(),
			))
			.service(handlers::set_cookie)
			.route("/game/", web::get().to(handlers::game_html))
			.service(handlers::game_files)
			.service(handlers::game_style)
			.service(handlers::game_audio)
			.service(handlers::flip)
			.service(handlers::flush)
			.service(handlers::redirect)
			.service(handlers::index)
			.service(handlers::index_files)
			.service(handlers::index_style)
			.service(handlers::count)
			.default_service(web::get().to(handlers::not_found))
	})
	.bind("0.0.0.0:8080")?
	.run()
	.await
}
