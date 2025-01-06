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
        div .vstack.gap-2 {
            ul .nav.flex-column {
                li .nav-item {
                    a .nav-link href="/" {
                        i .bi.bi-house-fill {} " Início"
                    }
                }
                li .nav-item {
                    a .nav-link href="/novo" {
                        i .bi.bi-pencil-fill {} " Novo"
                    }
                }
            }

            ul .nav.flex-column {
                li .nav-item {
                    a .nav-link href="/doações" {
                        i .bi.bi-box2-heart-fill {} " Doações"
                    }
                }
                li .nav-item {
                    a .nav-link href="/empréstimos" {
                        i .bi.bi-calendar2-week-fill {} " Empréstimos"
                    }
                }
                li .nav-item {
                    a .nav-link href="/trocas" {
                        i .bi.bi-arrow-repeat {} " Trocas"
                    }
                }
                li .nav-item {
                    a .nav-link href="/pedidos" {
                        i .bi.bi-person-raised-hand {} " Pedidos"
                    }
                }
            }


            // TODO: add auth logic
            ul .nav.flex-column {
                li .nav-item {
                    a .nav-link href="/entrar" {
                        i .bi.bi-lock-fill {} " Entrar"
                    }
                }
                li .nav-item {
                    a .nav-link href="/registrar" {
                        i .bi.bi-door-open-fill {} " Criar conta"
                    }
                }
                li .nav-item {
                    a .nav-link href="/minha-conta" {
                        i .bi.bi-person-fill {} " Minha conta"
                    }
                }
                li .nav-item {
                    a .nav-link href="/configurações" {
                        i .bi.bi-gear-fill {} " Configurações"
                    }
                }
            }

            // footer stuff
            small .text-muted.mb-3 {
                "Compartilhe objetos com colegas e ajude a criar uma universidade com menos consumo."
            }
            div .row.g-2.justify-content-center {
                div .col.text-center.my-auto {
                    a .text-decoration-none href="/política-de-privacidade" {
                        small { "Política de privacidade" }
                    }
                }
                div .col.text-center.my-auto {
                    a .text-decoration-none href="/termos-de-uso" {
                        small { "Termos de uso" }
                    }
                }
                div .col.text-center.my-auto {
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
