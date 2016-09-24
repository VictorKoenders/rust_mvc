use std::any::Any;
use std::error::Error;
use std::fmt;

#[macro_export]
macro_rules! view{
	() => {Ok($crate::view::ViewResultEnum::CurrentView)};
	($model: expr) => {Ok($crate::view::ViewResultEnum::CurrentViewWithModel(Box::new($model)))};
	($view:expr, $model:expr) => {Ok($crate::view::ViewResultEnum::SpecificViewWithModel(expr.to_string(), Box::new($model)))};
}

#[derive(Debug)]
pub struct ViewContext {
}

#[derive(Debug)]
pub enum ViewResultEnum {
	CurrentView,
	CurrentViewWithModel(Box<Any>),
	SpecificView(String),
	SpecificViewWithModel(String, Box<Any>)
}

pub type ViewResult = Result<ViewResultEnum, ViewError>;

#[derive(Debug)]
pub enum ViewError {
	UrlNotFound
}

impl fmt::Display for ViewError {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str(self.description())
	}
}

impl Error for ViewError {
	fn description(&self) -> &str {
		match *self {
			ViewError::UrlNotFound => "Url not found"
		}
	}
}


