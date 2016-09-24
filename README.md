# Rust MVC
An MVC framework written in rust.

For a working prototype, please see https://github.com/VictorKoenders/rust_mvc_example

Directory structure: 

![Structure](http://puu.sh/rmanA/68d87fda1c.png)

## Controllers

Controllers are files with several functions. If a function has the data annotation `#[http_url("/")]`, it will be visible in the website.

A controller has to return an rust_mvc::view::ViewResult. You can fill this ViewResult with the `view!` macro. There are several ways to call this:
- `view!()` returns the current view matching the fn name (`pub fn index() -> ViewResult { view!() }` will call the view 'index')
- `view!(model)` returns the current view, with the given model.
- `view!('some_view', model)` returns the specified view, with the given model.

To call a view without a model, simply call `view!('some_view', ())`.

See [controllers/index_controller.rs](https://github.com/VictorKoenders/rust_mvc_example/blob/master/src/controllers/index_controller.rs) for more info.

## Views

Views are simple HTML, with a couple of rules:

- At the top of the file, you can add #[use ...]. This will load the specific modules into the template
- At the top of the file, you can add #[model ...]. This will use the specific model for this view
- In the view itself, you can execute arbitrary code with #[...].

See [views/index2.html](https://github.com/VictorKoenders/rust_mvc_example/blob/master/src/views/index2.html) for more info
