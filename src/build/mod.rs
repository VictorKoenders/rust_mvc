mod parser;
mod data;
mod error;
mod generator;
mod config;

pub use build::config::Config;

pub fn build(config: Config) {
	let application = parser::parse(&config).unwrap();

	for view in &application.views {
		generator::generate_view_wrapper(view, &config);
	}
	generator::generate_view_mod(&application.views, &config);
	generator::generate_controller_mod(&application.controllers, &config);
	generator::generate_run(&application, &config);
}