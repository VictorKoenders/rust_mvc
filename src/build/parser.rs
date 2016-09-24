use std::error::Error;
use syntax::parse::token::{ DelimToken, Lit, Token };
use syntax::parse::{ ParseSess, parser, new_parser_from_file };
use std::path::{ Path, PathBuf };
use std::fs::{ File, read_dir };
use std::io::Read;

use build::data::*;
use build::error::ParseError;

pub use build::config::Config;

macro_rules! try_get {
	($e:expr, $d:expr) => {match $e { Some(o) => o, None => return Err(ParseError::UnexpectedNode($d.to_string())) }}
}

/// Load the ast from the application into memory and return it
/// This will generate the mod.rs and required mod.rs's,
/// parse the application and return the parsed AST
fn get_ast(file: &str) -> Vec<Token> {
	let session = ParseSess::new();
	let vec = Vec::new();
	let mut parser: parser::Parser = new_parser_from_file(&session, vec, Path::new(file));
	let mut result = Vec::new();

	loop {
		let t = parser.bump_and_get();

		if let Token::Eof = t { break; }
		else { result.push(t); }
	}
	result
}

fn find_attributes_reversed(ast: &Vec<Token>, end: usize) -> Vec<(String, Vec<String>)>{
	// attributes can either be:
	// - Pound, OpenDelim(Bracket), Ident(http_url#0), OpenDelim(Paren), Literal(Str_(/(60)), None), CloseDelim(Paren), CloseDelim(Bracket)
	// - Pound, OpenDelim(Bracket), Ident(http_url#0), OpenDelim(Paren), Literal(Str_(/api/planets/list(63)), None), CloseDelim(Paren), Comma, Ident(http_post#0), CloseDelim(Bracket)

	// attributes: [Ident(http_url#0), OpenDelim(Paren), Literal(Str_(/(60)), None), CloseDelim(Paren)]
	// attributes: [Ident(http_post#0)]

	// So we need to keep looking back until we find a token that is not any of these tokens:
	// - Pound
	// - OpenDelim
	// - Ident
	// - Literal
	// - CloseDelim
	// - Comma

	let mut start = end - 1;
	loop {
		match *ast.get(start).unwrap() {
			Token::OpenDelim(DelimToken::Bracket) | Token::OpenDelim(DelimToken::Paren) |
			Token::CloseDelim(DelimToken::Bracket) | Token::CloseDelim(DelimToken::Paren) |
			Token::Pound | Token::Literal(_, _) | Token::Ident(_) | Token::Comma => {
				if start == 0 { break; }
				start -= 1;
				continue
			},
			_ => {
				start += 1;
				break
			}
		}
	}

	find_attributes(ast, start, end)
}

/// Loop through the range of tokens and get all the attribute data
/// This assumes that the tokens are filtered by find_attributes_reversed
/// Will panic if one of the tokens is none of the tokens below:
/// - Token::OpenDelim(DelimToken::Bracket), Token::OpenDelim(DelimToken::Paren)
/// - Token::CloseDelim(DelimToken::Paren), Token::CloseDelim(DelimToken::Bracket)
/// - Literal
/// - Pound
/// - Ident
/// - Comma
fn find_attributes(ast: &Vec<Token>, start: usize, end: usize) -> Vec<(String, Vec<String>)> {
	let mut result = Vec::new();

	let mut name = String::new();
	let mut args = Vec::new();
	for i in start..end+1 {
		match *ast.get(i).unwrap() {
			Token::Pound => {},
			Token::OpenDelim(DelimToken::Bracket) | Token::OpenDelim(DelimToken::Paren) => {},
			Token::Comma | Token::CloseDelim(DelimToken::Paren) | Token::CloseDelim(DelimToken::Bracket) => {
				if name.len() > 0 { result.push((name, args)); }
				name = String::new();
				args = Vec::new();
			},
			Token::Ident(ident) => {
				if name.len() == 0 { name = ident.name.as_str().to_string(); }
					else { args.push(ident.name.as_str().to_string()); }
			},
			Token::Literal(lit, _) => {
				// TODO: Do something with the second argument (Option<String>) and with the other options of Lit:
				// https://doc.rust-lang.org/1.0.0/syntax/parse/token/enum.Lit.html
				match lit {
					Lit::Str_(n) => {
						if name.len() == 0 { name = n.as_str().to_string(); } else { args.push(n.as_str().to_string()); }
					},
					x => {
						panic!("Unknown lit: {:?}", x);
					}
				}
			},
			_ => panic!("Should never happen")
		}
	}

	result
}


fn parse_controller(application: &mut Application, ast: &Vec<Token>, name: String) -> Result<(), ParseError>{
	let mut offset = 0;
	let mut controller = Controller::new(name.to_string());
	loop {
		// find 'fn' position
		let position = offset + match ast.into_iter().skip(offset).position(|t| match *t { Token::Ident(ident) => ident.name.as_str().eq("fn"), _ => false }){
			Some(n) => n,
			None => break
		};
		offset = position + 1;

		// check if the previous statement is an Ident('pub')
		if position <= 0 { continue; }
		let ident = match *ast.get(position - 1).unwrap() {
			Token::Ident(ident) => ident,
			_ => continue
		};
		if !ident.name.as_str().eq("pub") { continue; }

		let name = match ast.get(position + 1) { Some(t) => t, None => continue };
		let name: String = match *name { Token::Ident(ident) => ident.name.as_str().to_string(), _ => continue };

		// get all attributes of this token, reversing back through the tokens
		let attr = find_attributes_reversed(ast, position - 2);

		let http_url = match attr.iter().find(|x| x.0.eq("http_url")) {
			Some(u) => {
				if u.1.len() > 0 { Some(u.1[0].clone()) }
				else { None }
			},
			None => None
		};
		let mut http_get = attr.iter().find(|x| x.0.eq("http_get")).is_some();
		let http_post = attr.iter().find(|x| x.0.eq("http_post")).is_some();
		let http_put = attr.iter().find(|x| x.0.eq("http_put")).is_some();
		let http_delete = attr.iter().find(|x| x.0.eq("http_delete")).is_some();

		// If we have an HTTP url, but none of the HTML methods
		// automatically set GET to true
		if let Some(url) = http_url {
			if !http_get && !http_post && !http_put && !http_delete {
				http_get = true;
			}
			let args = parse_controller_arguments(&ast, position + 2, &name);

			controller.actions.push(ControllerAction {
				name: name,
				path: url,
				allow_get: http_get,
				allow_put: http_put,
				allow_post:  http_post,
				allow_delete: http_delete,
				arguments: args,
			});
		}
	}

	if controller.actions.len() > 0 {
		application.controllers.push(controller);
	}

	Ok(())
}

fn parse_controller_arguments(ast: &Vec<Token>, start: usize, fn_name: &String) -> Vec<ControllerActionArgument> {
	// start should be an Token::OpenDelim(DelimToken::Paren)
	// then we need to find the matching Token::CloseDelim(DelimToken::Paren)
	// and just split the tokens by comma and stringify the name and type
	let mut depth = 0;
	let mut is_parsing_name = true;
	let mut name = String::new();
	let mut _type = String::new();
	let mut result: Vec<ControllerActionArgument> = Vec::new();

	for i in start..ast.len() {
		let str: String = match *ast.get(i).unwrap() {
			Token::OpenDelim(DelimToken::Paren) => {
				depth += 1;
				if depth == 1 { continue; }
				"(".to_string()
			},
			Token::CloseDelim(DelimToken::Paren) => {
				depth -= 1;
				if depth == 0 {
					if name.len() > 0 && _type.len() > 0 {
						result.push(ControllerActionArgument::new(name, _type));
					}
					return result;
				}
				")".to_string()
			},
			Token::Colon if depth == 1 => {
				is_parsing_name = false;
				continue;
			},
			Token::Comma if depth == 1 => {
				is_parsing_name = true;
				result.push(ControllerActionArgument::new(name, _type));
				name = String::new();
				_type = String::new();
				continue;
			},
			Token::Colon => ":".to_string(),
			Token::Comma => ",".to_string(),
			Token::Lt => "<".to_string(),
			Token::Gt => ">".to_string(),
			Token::Underscore => "_".to_string(),
			Token::Ident(ref name) => name.name.as_str().to_string(),
			ref x => panic!("Unknown type {:?}", x)
		};
		if is_parsing_name {
			name.push_str(&str);
		} else {
			_type.push_str(&str);
		}
	}
	println!("{:?}", ast.into_iter().skip(start).collect::<Vec<&Token>>());

	panic!("Unexpected end of fn {}", fn_name);
}

fn parse_view(file: PathBuf) -> Result<View, ParseError> {
	let mut file_contents: String = {
		let mut f = try!(File::open(&file).map_err(|e| ParseError::FileError(e.description().to_string())));
		let mut s = String::new();
		try!(f.read_to_string(&mut s).map_err(|e| ParseError::FileError(e.description().to_string())));
		s
	};

	let mut view = View::new(format!("{}", file.file_stem().unwrap().to_str().unwrap()));

	loop {
		let mut clone = {
			let index = match file_contents.find("#[") {
				Some(i) => i,
				None => {
					if file_contents.trim().len() > 0 {
						view.parts.push(ViewPart::Static(file_contents));
					}
					break
				}
			};
			let (part, remaining) = file_contents.split_at(index);
			if part.trim().len() > 0 {
				view.parts.push(ViewPart::Static(part.to_string()));
			}
			remaining.to_string()
		};

		let part = {
			let mut depth = 0;
			let mut string_token = '\0';
			let mut buffer = String::new();

			for char in clone.chars() {
				buffer.push(char);

				if char == '\'' || char == '"' {
					if string_token == '\0' { string_token = char;}
					else if string_token == char { string_token = '\0';}
					continue;
				}
				if char == '[' {
					depth += 1;
					continue;
				}
				if char == ']' {
					depth -= 1;
					if depth == 0 {
						break;
					}
				}
			}

			buffer
		};
		if part.len() == 0 { break; }
		clone.drain(..part.len());
		file_contents = clone;

		let part = part[2..part.len() - 1].to_string();

		if view.parts.iter().all(|x| if let &ViewPart::Code(_) = x { true } else { false }) {
			// we have no static yet
			// This part might be either:
			// - use xxx
			// - model xxx
			if part.starts_with("use ") {
				view.use_namespaces.push(part[4..].to_string());
			}
			if part.starts_with("model ") {
				view.model = Some(part[6..].to_string());
			}
		} else {
			view.parts.push(ViewPart::Code(part));
		}
	}

	Ok(view)
}

pub fn parse(config: &Config) -> Result<Application, ParseError> {
	let mut application = Application::new();
	println!("Dir: {}/{}", config.root_dir, config.controller_dir);
	for path in read_dir(&format!("{}/{}", config.root_dir, config.controller_dir)).unwrap() {
		// TODO: Error check this
		let path = path.unwrap().path();
		let name = path.file_stem().unwrap().to_str().unwrap().to_string();
		if !name.ends_with("_controller") { continue; }
		let name = name[..name.len() - "_controller".len()].to_string();
		println!("Loading {} ({})", name, path.to_str().unwrap());
		let ast = get_ast(path.to_str().unwrap());
		try!(parse_controller(&mut application, &ast, name));
	}
	for path in read_dir("src/views").unwrap() {
		// TODO: Error check this
		let path = path.unwrap().path();
		if path.extension().unwrap() == "html" {
			let view = try!(parse_view(path));
			application.views.push(view);
		}
	}

	Ok(application)
}