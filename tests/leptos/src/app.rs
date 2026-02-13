use ftml_components::{config::FtmlConfig, SidebarPosition};
use ftml_dom::toc::TocSource;
use ftml_ontology::utils::Css;
use ftml_uris::DocumentUri;
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
            //<Scrollbar style="width:100vw;height:100vh;">
                <main>
                    <HomePage/>
                    //<Routes fallback=|| "Page not found.".into_view()>
                    //    <Route path=StaticSegment("") view=HomePage/>
                    //</Routes>
                </main>
            //</Scrollbar>
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

fn extract_body_as_div(s: &str) -> String {
    let i = s.find("<body").expect("exists");
    let s = &s[i + "<body".len()..];
    let i = s.rfind("/body>").expect("exists");
    let s = &s[..i];
    format!("<div{s}/div>")
}
fn extract_body(s: &str) -> String {
    let i = s.find("<body").expect("exists");
    let s = &s[i + "<body".len()..];
    let i = s.find('>').expect("exists");
    let s = &s[i + 1..];
    let i = s.rfind("</body>").expect("exists");
    let s = &s[..i];
    s.to_string()
}

macro_rules! backend {
    ($num:literal:[$($name:literal),*]) => {

        ftml_backend::new_global!(GlobalBackend = Cached(RemoteFlamsLike [
            $(
                concat!("https://mathhub.info?a=FTML/meta&p=tests&d=",$name,"&l=en")
                => concat!("http://localhost:3000/api/get?d=",$name,".en")
            ),*
        ;$num]));

        #[server(
          prefix="/api",
          endpoint="get",
          input=server_fn::codec::GetUrl,
          output=server_fn::codec::Json
        )]
        async fn get(d: String) -> Result<(DocumentUri, Vec<Css>, String), ServerFnError<String>> {
            fn go(uri: &str, s: &str) -> Result<(DocumentUri, Vec<Css>, String), ServerFnError<String>> {

                Ok((
                    format!("https://mathhub.info?a=FTML/meta&p=tests&d={uri}&l=en")
                        .parse()
                        .expect("is valid"),
                    Vec::new(),
                    extract_body(s),
                ))
            }
            match d.as_str() {
                $(
                    concat!($name,".en") => go($name,include_str!(concat!("../public/",$name,".en.html"))),
                )*
                _ => Err("nope".to_string().into()),
            }
        }
    };
}
backend!(10: ["sections","para","symbolsmodules","paragraphs","structures","morphisms","slides","metatheory","problems","proof"]);

type Views = ftml_components::Views<GlobalBackend>;

#[component]
fn Ftml() -> impl IntoView {
    use ftml_dom::FtmlViews;
    let uri: ftml_uris::DocumentUri = "https://mathhub.info?a=FTML/meta&p=tests&d=all&l=en"
        .parse()
        .unwrap();
    const HTML: &str = include_str!("../public/all.en.html");
    let html = extract_body_as_div(HTML);
    tracing::info!("Here");
    //FtmlConfig::set_toc_source(TocSource::Extract);
    Views::setup_document::<GlobalBackend>(
        uri,
        SidebarPosition::Find,
        false,
        TocSource::Extract,
        || Views::render_ftml(html, None).into_any(),
    )
}
