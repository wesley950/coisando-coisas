use coisando_coisas::LocalUser;
use maud::html;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub deslocamento: Option<usize>,
    pub quantidade: Option<usize>,
}

pub mod components;

fn render_base(content: maud::Markup, local_user: LocalUser) -> maud::Markup {
    html! {
        html lang="pt-br" {
            head {
                meta charset="utf-8";
                // TODO: add favicon
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Coisando Coisas" }
                link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH" crossorigin="anonymous";
                script src="https://kit.fontawesome.com/abfe2b7043.js" crossorigin="anonymous" {}
            }
            body {
                (components::render_navbar())

                div .container.mt-4.mb-4 {
                    div .row {
                        // left menu, visible on md and lg screens
                        div .col-md-4.col-lg-3.d-none.d-md-block {
                            (components::render_menu(&local_user))
                        }

                        // collapsible menu, visible on sm and xs screens
                        div .col-md-4.col-lg-3.mb-4.collapse.d-md-none #menu {
                            (components::render_menu(&local_user))
                        }

                        // main content
                        div .col {
                            (content)
                        }
                    }
                }

                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}
            }
        }
    }
}

pub mod auth;
pub mod index;
pub mod info;
pub mod submit;
