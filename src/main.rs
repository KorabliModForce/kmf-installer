use std::path::PathBuf;

use component::{Content, StageSideBar};
use dioxus::{
  desktop::{Config, WindowBuilder},
  prelude::*,
};
use url::Url;

mod component;

// const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
  dioxus::LaunchBuilder::desktop()
    .with_cfg(
      Config::default()
        .with_menu(None)
        .with_window(WindowBuilder::new().with_title("KMF Frontier")),
    )
    .launch(App)
}

#[derive(Clone, Copy)]
struct GlobalEvents {
  onreset: EventHandler<()>,
  onstep: EventHandler<()>,
  onselectgame: EventHandler<PathBuf>,
  onselectsource: EventHandler<Url>,
  onselectmods: EventHandler<Vec<String>>,
}

#[derive(Clone)]
struct InstallContext {
  mods: Vec<String>,
  game_path: PathBuf,
  source: Url,
}

#[component]
fn App() -> Element {
  let mut stage = use_signal(|| 0);
  let mut install_context = use_signal(|| InstallContext {
    mods: Vec::new(),
    game_path: PathBuf::from(""),
    source: Url::parse("https://kmf-station.zice.top").expect("it should be ok"),
  });

  use_context_provider(|| GlobalEvents {
    onreset: EventHandler::new(move |_| {
      stage.set(0);
    }),
    onstep: EventHandler::new(move |_| {
      *stage.write() += 1;
    }),
    onselectgame: EventHandler::new(move |path: PathBuf| {
      install_context.write().game_path = path;
    }),
    onselectsource: EventHandler::new(move |url| {
      install_context.write().source = url;
    }),
    onselectmods: EventHandler::new(move |mods| {
      install_context.write().mods = mods;
    }),
  });

  use_context_provider(|| install_context);

  rsx! {
    // document::Link { rel: "icon", href: FAVICON }
    document::Link { rel: "stylesheet", href: MAIN_CSS }
    div { class: "main-container",
      StageSideBar { current: stage() }
      Content { current: stage() }
    }
  }
}
