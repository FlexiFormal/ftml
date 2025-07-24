use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <Stylesheet href="https://raw.githack.com/Jazzpirate/RusTeX/main/rustex/src/resources/rustex.css"/>
                <Stylesheet href="https://cdn.jsdelivr.net/gh/dreampulse/computer-modern-web-font@master/font/Typewriter/cmun-typewriter.css"/>
                <Stylesheet href="https://cdn.jsdelivr.net/gh/dreampulse/computer-modern-web-font@master/font/Serif/cmun-serif.css"/>
                <Stylesheet href="https://fonts.cdnfonts.com/css/latin-modern-math"/>
                <Stylesheet href="https://cdn.jsdelivr.net/gh/dreampulse/computer-modern-web-font@master/font/Sans/cmun-sans.css"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    use ftml_dom::FtmlViews;
    use ftml_leptos::Views;
    use thaw::Scrollbar;
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/axum-test.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        //<Router>
            <Scrollbar style="width:100vw;height:100vh;">
                <main>
                    <HomePage/>
                    //<Routes fallback=|| "Page not found.".into_view()>
                    //    <Route path=StaticSegment("") view=HomePage/>
                    //</Routes>
                </main>
            </Scrollbar>
        //</Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"Welcome to Leptos!"</h1>
        <Ftml/>
    }
}

#[component]
fn Ftml() -> impl IntoView {
    use ftml_dom::FtmlViews;
    let uri: ftml_uris::DocumentUri = "https://mathhub.info?a=Papers/25-CICM-FLAMS&d=paper&l=en"
        .parse()
        .unwrap();
    const HTML: &str = include_str!("test.html");
    ftml_leptos::Views::top(|| {
        ftml_dom::setup_document(uri, || ftml_leptos::Views::render_ftml(HTML.to_string()))
    })
}
