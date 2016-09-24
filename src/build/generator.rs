use build::data::{ Application, Controller, View, ViewPart };
use std::fs::File;
use std::io::Write;
use build::config::Config;

const GENERATE_MESSAGE: &'static str = r#"/// THIS IS AN AUTO_GENERATED FILE, DO NOT MODIFY
/// Any an all changes that you make here will be removed on the next build
/// For more info, visit the rust_mvc crate description at http://github.com/victorkoenders/rust_mvc

"#;

pub fn generate_view_wrapper(view: &View, config: &Config){
	let mut result = String::new();

	result.push_str(GENERATE_MESSAGE);

	for use_namespace in &view.use_namespaces {
		result.push_str(&format!("use {};\r\n", use_namespace));
	}
	result.push_str("\r\n");

	result.push_str("pub fn generate(");
	result.push_str(match view.model {
		Some(ref model) => format!("model: &{}", model),
		None => "".to_string()
	}.as_str());
	result.push_str(") -> String {\r\n\tlet mut result = String::new();\r\n");
	for part in &view.parts {
		match part {
			&ViewPart::Static(ref str) => {
				result.push_str("\tresult.push_str(r#\"");
				result.push_str(str.as_str());
				result.push_str("\"#);\r\n")
			},
			&ViewPart::Code(ref str) => {
				result.push_str("\tresult.push_str(&format!{\"{}\",");
				result.push_str(str);
				result.push_str("});\r\n");
			}
		}
	}
	result.push_str("\tresult\r\n}");

	let file = format!("{}/{}/{}.rs", config.root_dir, config.view_dir, view.name);
	let mut f = File::create(&file).unwrap();
	f.write_all(result.as_bytes()).unwrap();
	drop(f);
}
pub fn generate_view_mod(view: &Vec<View>, config: &Config){
	let mut output = String::new();
	output.push_str(GENERATE_MESSAGE);
	for view in view {
		output.push_str(&format!("pub mod {};\r\n", view.name));
	}

	let file = format!("{}/{}/mod.rs", config.root_dir, config.view_dir);
	let mut f = File::create(&file).unwrap();
	f.write_all(output.as_bytes()).unwrap();
}

pub fn generate_controller_mod(controllers: &Vec<Controller>, config: &Config){
	let mut output = String::new();
	output.push_str(GENERATE_MESSAGE);
	for controller in controllers {
		output.push_str(&format!("pub mod {}_controller;\r\n", controller.name));
	}

	let file = format!("{}/{}/mod.rs", config.root_dir, config.controller_dir);
	let mut f = File::create(&file).unwrap();
	f.write_all(output.as_bytes()).unwrap();
}

pub fn generate_run(application: &Application, config: &Config){
	let mut output = String::new();
	output.push_str(GENERATE_MESSAGE);

	output.push_str("#[allow(unused_imports)] use controllers;\r\n");
	output.push_str("#[allow(unused_imports)] use views;\r\n");
	output.push_str("#[allow(unused_imports)] use rust_mvc;\r\n\r\n");

	let mut namespaces_included: Vec<String> = Vec::new();
	for view in &application.views {
		for use_namespace in &view.use_namespaces {
			if namespaces_included.iter().any(|x| x.eq(use_namespace)) { continue; }

			output.push_str(&format!("use {};\r\n", use_namespace));
			namespaces_included.push(use_namespace.clone());
			println!("{:?}", namespaces_included);
		}
	}

	output.push_str("use std::any::Any;\r\n\r\n");

	output.push_str("#[allow(dead_code)] fn resolve_url(request: &mut rust_mvc::server::Request) -> (&'static str, rust_mvc::view::ViewResult) {\r\n");
	output.push_str("\tlet view_context: rust_mvc::view::ViewContext = request.get_view_context();\r\n");
	for controller in &application.controllers {
		for action in &controller.actions {
			output.push_str(&format!("\tif request.url_match(\"{}\") {{ return (\"{}\", controllers::{}_controller::{}(view_context)); }}\r\n",
									 action.path, action.name, controller.name, action.name));
		}
	}
	output.push_str("\t(\"\", Err(rust_mvc::view::ViewError::UrlNotFound))\r\n");
	output.push_str("}\r\n\r\n");

	output.push_str("#[allow(dead_code)] fn execute_view(view_name: String) -> Option<String> {\r\n");
	for view in &application.views {
		if view.model.is_some() { continue; }
		output.push_str(&format!("\tif view_name.eq(\"{}\") {{ return Some(views::{}::generate()); }}\r\n", view.name, view.name));
	}
	output.push_str("\tNone\r\n");
	output.push_str("}\r\n\r\n");

	output.push_str("#[allow(dead_code)] fn execute_view_with_model(view_name: String, model: Box<Any>) -> Option<String> {\r\n");
	for view in &application.views {
		if view.model.is_none() { continue; }
		let ref model = view.model.as_ref().unwrap();
		//" return Some(views::{}::generate(})); }}\r\n", view.name, view.name, model));
		output.push_str(&format!("\tif view_name.eq(\"{}\") {{\r\n", view.name));
		output.push_str(&format!("\t\tif let Some(model) = model.downcast_ref::<{}>() {{\r\n", model));
		output.push_str(&format!("\t\t\treturn Some(views::{}::generate(model));\r\n", view.name));
		output.push_str(&format!("\t\t}} else {{\r\n"));
		output.push_str(&format!("\t\t\treturn None;\r\n"));
		output.push_str(&format!("\t\t}}\r\n"));
		output.push_str(&format!("\t}}\r\n"));

	}
	output.push_str("\tNone\r\n");
	output.push_str("}\r\n\r\n");

	output.push_str("#[allow(dead_code)] pub fn run() -> rust_mvc::server::Server {\r\n");
	output.push_str("\trust_mvc::server::Server::create(\"localhost\", 8181, resolve_url, execute_view, execute_view_with_model)\r\n");
	//output.push_str("\tserver.listen();\r\n");

	output.push_str("}\r\n");

	let file = format!("{}/run.rs", config.root_dir);
	let mut f = File::create(&file).unwrap();
	f.write_all(output.as_bytes()).unwrap();
}