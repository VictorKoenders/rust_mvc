use view::{ ViewResult, ViewResultEnum };
use std::any::Any;
use hyper::server::{ Server as HyperServer, Request as HyperRequest, Response as HyperResponse, Listening };
use std::marker::{ Send, Sync };

use server::Request;
use std::error::Error;

pub struct Server {
	server: Listening
}

impl Server {
	pub fn create<
		THost: ToString,
		TControllerCallback: Send + Sync + 'static + Fn(&mut Request) -> (&'static str, ViewResult),
		TViewCallback: Send + Sync + 'static + Fn(String) -> Option<String>,
		TViewModelCallback: Send + Sync + 'static + Fn(String, Box<Any>) -> Option<String>
	>(host: THost, port: u16, controller_callback: TControllerCallback, view_callback: TViewCallback, view_model_callback: TViewModelCallback) -> Server {
		let addr = format!("{}:{}", host.to_string(), port);
		Server {
			server: HyperServer::http(addr.as_str()).unwrap().handle(move |req: HyperRequest, res: HyperResponse| {
				let url = format!("{}", req.uri);
				let mut request = Request::new(url.clone());

				let result = controller_callback(&mut request);
				let result = (result.0, match result.1 {
					Err(e) => {
						res.send(e.description().as_bytes()).unwrap();
						return;
					},
					Ok(r) => r
				});

				let view_result = match result.1 {
					ViewResultEnum::CurrentView => view_callback(result.0.to_string()),
					ViewResultEnum::CurrentViewWithModel(model) => view_model_callback(result.0.to_string(), model),
					ViewResultEnum::SpecificView(view) => view_callback(view),
					ViewResultEnum::SpecificViewWithModel(view, model) => view_model_callback(view, model),
				};
				
				match view_result {
					Some(r) => res.send(r.as_bytes()).unwrap(),
					None => res.send(b"<html><body><h1>Page not found</h1></body></html>").unwrap()
				};
			}).unwrap()
		}
	}

	#[allow(dead_code)]
	pub fn close(&mut self) {
		self.server.close().unwrap();
	}
}
