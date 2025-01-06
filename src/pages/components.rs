use maud::html;

pub fn render_navbar() -> maud::Markup {
    html! {
        nav .navbar.bg-primary.navbar-dark.sticky-top {
            div .container {
                a .navbar-brand.mx-auto href="/" {
                    i .bi.bi-box2-heart-fill {} " Coisando Coisas"
                }
                button .btn.btn-primary.d-block.d-md-none type="button" data-bs-toggle="collapse" data-bs-target="#menu" aria-expanded="false" aria-controls="menu" {
                    i .bi.bi-list {} " Menu"
                }
            }
        }
    }
}

pub fn render_menu() -> maud::Markup {
    html! {
        div .vstack.gap-3 {
            div .list-group.shadow {
                a .list-group-item.list-group-item-action.active href="/" {
                    i .bi.bi-house-fill {} " Início"
                }
                a .list-group-item.list-group-item-action href="/novo" {
                    i .bi.bi-pencil-fill {} " Novo"
                }
            }

            div .list-group.shadow {
                a .list-group-item.list-group-item-action href="/doações" {
                    i .bi.bi-box2-heart-fill {} " Doações"
                }
                a .list-group-item.list-group-item-action href="/empréstimos" {
                    i .bi.bi-calendar2-week-fill {} " Empréstimos"
                }
                a .list-group-item.list-group-item-action href="/trocas" {
                    i .bi.bi-arrow-repeat {} " Trocas"
                }
                a .list-group-item.list-group-item-action href="/pedidos" {
                    i .bi.bi-person-raised-hand {} " Pedidos"
                }
            }

            // TODO: add auth logic
            div .list-group.shadow {
                a .list-group-item.list-group-item-action href="/entrar" {
                    i .bi.bi-lock-fill {} " Entrar"
                }
                a .list-group-item.list-group-item-action href="/registrar" {
                    i .bi.bi-door-open-fill {} " Criar conta"
                }
                a .list-group-item.list-group-item-action href="/minha-conta" {
                    i .bi.bi-person-fill {} " Minha conta"
                }
                a .list-group-item.list-group-item-action href="/configurações" {
                    i .bi.bi-gear-fill {} " Configurações"
                }
            }

            // footer stuff
            small .text-muted {
                "Compartilhe objetos com colegas e ajude a criar uma universidade com menos consumo."
            }
            div .row.g-2 {
                div .col.text-center {
                    a .text-decoration-none href="/política-de-privacidade" {
                        small { "Política de privacidade" }
                    }
                }
                div .col.text-center {
                    a .text-decoration-none href="/termos-de-uso" {
                        small { "Termos de uso" }
                    }
                }
                div .col.text-center {
                    a .text-decoration-none href="/sobre" {
                        small { "Sobre" }
                    }
                }
            }
            div .row {
                a .text-decoration-none.text-center href="https://github.com/wesley950/coisando-coisas" {
                    small {
                        i .bi.bi-github {} " Código fonte"
                    }
                }
            }
        }
    }
}
