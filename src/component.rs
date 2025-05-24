use std::{collections::HashSet, path::PathBuf};

use dioxus::prelude::*;
use directories::ProjectDirs;
use native_dialog::DialogBuilder;
use tokio::fs;
use tracing::debug;
use url::Url;

use crate::{GlobalEvents, InstallContext};

#[derive(Debug, PartialEq, Clone)]
pub struct Mod {
  id: String,
  name: String,
  description: String,
}

#[component]
pub fn InstallCompleted() -> Element {
  rsx! {
    "安装完成！现在你可以关闭窗口了"
  }
}

#[component]
pub fn InstallMods() -> Element {
  let events = use_context::<GlobalEvents>();
  let install_context = use_context::<Signal<InstallContext>>();
  let kmf_result = use_resource(move || async move {
    debug!(
      "game is file:{}",
      install_context().game_path.to_string_lossy()
    );
    let kmf_instance = kmf::Kmf::try_from_config(&kmf::Config {
      cache_dir: ProjectDirs::from("com", "zerodegress", "kmf-installer")
        .expect("it should be ok")
        .cache_dir()
        .to_path_buf(),
      default_game: Some(format!(
        "file:{}",
        install_context().game_path.to_string_lossy()
      )),
      progress_draw_target: kmf::config::ProgressDrawTargetType::Hidden,
    })
    .await;

    match kmf_instance {
      Err(err) => Err(anyhow::Error::from(err)),
      Ok(kmf_instance) => {
        let res = kmf_instance
          .run(kmf::task::Task::Install {
            url: install_context()
              .mods
              .iter()
              .map(|x| Url::parse(format!("kmf:{x}").as_str()).expect("it should be ok"))
              .collect(),
            game: None,
          })
          .await;
        if res.is_ok() {
          events.onstep.call(());
        }
        res.map_err(anyhow::Error::from)
      }
    }
  });

  rsx! {
    match &*kmf_result.value().read() {
      None => rsx! {
        "正在安装中，请坐和放宽"
      },
      Some(x) => match x {
        Err(err) => rsx! {
          "出现错误:{err}"
        },
        Ok(()) => rsx! {}
      }
    }
  }
}

#[component]
pub fn SelectMods() -> Element {
  let events = use_context::<GlobalEvents>();
  let mods = use_signal(|| {
    vec![Mod {
      id: "korabli-lesta-l10n".to_string(),
      name: "澪刻战舰世界汉化".to_string(),
      description: "基本汉化包".to_string(),
    }]
  });
  let mut selected_mods = use_signal(HashSet::new);
  let mut focus_mod = use_signal(|| None);

  rsx! {
    div {
      class: "select-mod",
      div {
        class: "mods-sidebar",
        {mods().into_iter().map(|x| rsx! {
          div {
            class: "mods-sidebar-item".to_string() + if selected_mods().contains(x.id.as_str()) { " checked" } else { "" },
            input {
              id: format!("select-mod-{}", x.id.as_str()),
              type: "checkbox",
              onchange: {
                let x = x.to_owned();
                move |ev: Event<FormData>| {
                  if ev.checked() {
                    *focus_mod.write() = Some(x.id.to_owned());
                    selected_mods.write().insert(x.id.to_owned());
                  } else {
                    if focus_mod().is_some_and(|y| y == x.id) {
                    *focus_mod.write() = None;
                    }
                    selected_mods.write().remove(x.id.as_str());
                  }
                }
              },
            },
            label {
              r#for: format!("select-mod-{}", x.id.as_str()),
              {x.name.as_str()}
            },
          }
        })}
      },
      div {
        class: "mods-content",
        {focus_mod().map(|x| {
          let m = mods().iter().find(|y| y.id == x).expect("here comes Mod").to_owned();
          rsx! {
            h3 { {m.name} },
            p { {m.description} },
          }
        })}
      },
    }
    if !selected_mods().is_empty() {
      Row {
        reverse: true,
        button {
          onclick: move |_| {
            events.onselectmods.call(selected_mods().iter().cloned().collect());
            events.onstep.call(());
          },
          "下一步"
        },
      }
    }
  }
}

#[component]
pub fn SelectSource() -> Element {
  let events = use_context::<GlobalEvents>();
  let mut source = use_signal(|| None);

  rsx! {
    div { class: "select-source",
      fieldset {
        onchange: move |ev| {
          tracing::debug!("{:?}", ev);
          if ev.value().as_str() == "kmf-station" { source.set(Some(Url::parse("https://kmf-station.zice.top").expect("it should be ok"))) }
        },
        div { class: "source",
          input {
            type: "radio",
            id: "select-source-kmf-station",
            name: "select-source",
            value: "kmf-station",
          },
          label {
            r#for: "select-source-kmf-station",
            "KMF Station"
          },
        },
      }
    }
    if source().is_some() {
      Row {
        reverse: true,
        button {
          onclick: move |_| {
            let Some(source) = source() else {
              return;
            };
            events.onselectsource.call(source);
            events.onstep.call(());
          },
          "下一步"
        }
      }
    }
  }
}

#[component]
pub fn SelectGame() -> Element {
  let mut selected_path = use_signal(|| "".to_string());
  let mut selected_path_error = use_signal(|| Some("正在检查"));
  let events = use_context::<GlobalEvents>();
  let install_context = use_context::<Signal<InstallContext>>();

  use_effect(move || {
    let selected_path = selected_path().to_owned();
    if selected_path.as_str() == "" {
      return;
    }
    spawn(async move {
      selected_path_error.set(match fs::try_exists(selected_path.to_string()).await {
        Err(_err) => Some("错误"),
        Ok(_) => {
          events.onselectgame.call(PathBuf::from(selected_path));
          debug!(
            "game is file:{}",
            install_context().game_path.to_string_lossy()
          );
          None
        }
      });
    });
  });

  rsx! {
    div { class: "select-game",
      input {
        type: "text",
        value: selected_path,
        placeholder: "选择游戏安装目录...",
        readonly: true,
      }
      button {
        onclick: move |_| {
          spawn(async move {
            let Some(path) = DialogBuilder::file()
              .set_location("~")
              .open_single_dir()
              .show()
              .unwrap() else {
              return;
            };
            selected_path.set(path.to_string_lossy().into())
          });
        },
        "浏览目录"
      }
    }
    Row { reverse: true,
      if selected_path_error().is_none() {
        button {
          onclick: move |_| {
            events.onstep.call(());
          },
          "下一步"
        }
      }
    }
  }
}

#[derive(Debug, Props, PartialEq, Clone, Copy)]
pub struct ContentProps {
  current: usize,
}

#[component]
pub fn Content(props: ContentProps) -> Element {
  let events = use_context::<GlobalEvents>();

  rsx! {
    main { class: "content-area",
      div { class: "stage-content",
        {
          match props.current {
            0 => rsx! {
              h2 { "选择游戏安装目录" }
              SelectGame {}
            },
            1 => rsx! {
              h2 { "选择安装源" }
              SelectSource {}
            },
            2 => rsx! {
              h2 { "选择要安装的模组" }
              SelectMods {}
            },
            3 => rsx! {
              h2 { "安装中" }
              InstallMods {}
            },
            4 => rsx! {
              h2 { "安装完成" }
              InstallCompleted {}
            },
            _ => rsx! {
              h2 { "你不该到这里的" }
              button {
                onclick: move |_| {
                    events.onreset.call(());
                },
                "重置"
              }
            },
          }
        }
      }
    }
  }
}

#[derive(Debug, Props, PartialEq, Clone, Copy)]
pub struct StageSideBarProps {
  current: usize,
}

#[component]
pub fn StageSideBar(props: StageSideBarProps) -> Element {
  let items = [
    "选择安装目录",
    "选择安装源",
    "选择模组",
    "安装中",
    "安装完成",
  ];

  let stageClass = |stage: usize| {
    if stage == props.current {
      "stage-item active"
    } else {
      "stage-item"
    }
  };

  rsx! {
    Col { class: "stage-sidebar",
      {items.iter().enumerate().map(|(i, x)| rsx! {
        Row { class: stageClass(i),
          div { class: "stage-item-icon", "{i + 1}" }
          "{x}"
        }
      })}
    }
  }
}

#[derive(Debug, Props, PartialEq, Clone)]
pub struct RowProps {
  class: Option<String>,
  reverse: Option<bool>,
  children: Element,
}

#[component]
pub fn Row(props: RowProps) -> Element {
  let class = || {
    format!(
      "{} row {}",
      props.class.unwrap_or_default(),
      if props.reverse.is_some_and(|x| x) {
        "reverse"
      } else {
        ""
      }
    )
  };

  rsx! {
    div { class: class(), {props.children} }
  }
}

#[derive(Debug, Props, PartialEq, Clone)]
pub struct ColumnProps {
  class: Option<String>,
  reverse: Option<bool>,
  children: Element,
}

#[component]
pub fn Col(props: ColumnProps) -> Element {
  let class = || {
    format!(
      "{} column {}",
      props.class.unwrap_or_default(),
      if props.reverse.is_some_and(|x| x) {
        "reverse"
      } else {
        ""
      }
    )
  };

  rsx! {
    div { class: class(), {props.children} }
  }
}
