# GTK Webby

Webby is a proof-of-concept application that behaves like a browser, but one that renders native GTK
applications rather than websites.

## Running

To run, first make sure you have the following installed:

1. Rust + Cargo
2. GTK4 development libraries
3. Lua 5.4 development libraries

An easy way to do this is to use `nix-shell`, or you can use your package manager.

Once these are installed, running is simply

```sh
$ cargo run
```

![Screenshot](images/screenshot.png)

### Examples

Running Webby will launch a window that can be used to load GTK "web" applications. A number of
examples are included with Webby that only require Rust:

```sh
$ cd examples/hello
$ cargo run
```

Once the example is running, you can load it in Webby by using the URL `http://localhost:8000`.

## Tips

When running the app, use `Ctrl-Shift-D` to open up the GTK inspector.

<!-- vim: set tw=100: -->
