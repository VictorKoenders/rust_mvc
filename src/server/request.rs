use view::ViewContext;

pub struct Request {
	url: String,
}


impl Request {
	pub fn new(url: String) -> Request {
		Request {
			url: url
		}
	}
	pub fn get_view_context(&self) -> ViewContext {
		ViewContext {

		}
	}

	pub fn url_match(&self, url: &'static str) -> bool {
		url.eq(self.url.as_str())
	}
}