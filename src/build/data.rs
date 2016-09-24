pub struct Application {
	pub controllers: Vec<Controller>,
	pub views: Vec<View>,
}

impl Application {
	pub fn new() -> Application { Application { controllers: Vec::new(), views: Vec::new() }}
}

pub struct Controller {
	pub name: String,
	pub actions: Vec<ControllerAction>,
}

impl Controller {
	pub fn new(name: String) -> Controller { Controller { name: name, actions: Vec::new() }}
}

pub struct ControllerAction {
	pub name: String,
	pub path: String,
	pub allow_get: bool,
	pub allow_put: bool,
	pub allow_post: bool,
	pub allow_delete: bool,
	pub arguments: Vec<ControllerActionArgument>,
}

pub struct ControllerActionArgument {
	pub name: String,
	pub _type: String,
}

impl ControllerActionArgument {
	pub fn new(name: String, _type: String) -> ControllerActionArgument { ControllerActionArgument { name: name, _type: _type }}
}

pub struct View {
	pub name: String,
	pub model: Option<String>,
	pub use_namespaces: Vec<String>,
	pub parts: Vec<ViewPart>,
}

impl View {
	pub fn new(name: String) -> View { View { name: name, model: None, parts: Vec::new(), use_namespaces: Vec::new() }}
}

pub enum ViewPart {
	Static(String),
	Code(String)
}
