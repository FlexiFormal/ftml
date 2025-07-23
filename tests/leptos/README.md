# SSR Example

This example has a server function (`get_some_html` in [app.rs](./src/app.rs)) that returns an HTML string. `SomeComplicatedComponent` (same file) calls the server function, inserts the string in the DOM and then runs over the newly generated nodes, replacing all DOM nodes with attribute `data-replace-with-leptos` with a red border and a thaw PopOver on hover.

To run, use e.g. `cargo leptos watch --release`