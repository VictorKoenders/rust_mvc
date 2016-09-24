pub struct Config {
	pub root_dir: String,
	pub controller_dir: String,
	pub view_dir: String,
}

impl Config {
	pub fn new() -> Config {
		Config {
			root_dir: "".to_string(),
			controller_dir: "controllers".to_string(),
			view_dir: "views".to_string()
		}
	}
}