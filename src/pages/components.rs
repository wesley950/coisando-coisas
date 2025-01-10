use coisando_coisas::LocalUser;
use maud::html;

pub fn render_navbar() -> maud::Markup {
    html! {
        nav .navbar.bg-primary.navbar-dark.sticky-top {
            div .container {
                a .navbar-brand.mx-auto href="/" {
                    i .fa-solid.fa-hand-holding-heart {}
                    strong { " Coisando Coisas" }
                }
                button .btn.btn-primary.d-block.d-md-none type="button" data-bs-toggle="collapse" data-bs-target="#menu" aria-expanded="false" aria-controls="menu" {
                    i .fa-solid.fa-caret-down {} " Menu"
                }
            }
        }
    }
}

pub fn render_menu(local_user: &LocalUser) -> maud::Markup {
    html! {
        div .vstack.gap-2 {
            ul .nav.flex-column {
                li .nav-item {
                    a .nav-link href="/" {
                        i .fa-solid.fa-house {} " Início"
                    }
                }
                li .nav-item {
                    a .nav-link href="/novo" {
                        i .fa-solid.fa-pen-to-square {} " Novo"
                    }
                }
            }

            ul .nav.flex-column {
                li .nav-item {
                    a .nav-link href="/doações" {
                        i .fa-solid.fa-gift {} " Doações"
                    }
                }
                li .nav-item {
                    a .nav-link href="/empréstimos" {
                        i .fa-solid.fa-clock {} " Empréstimos"
                    }
                }
                li .nav-item {
                    a .nav-link href="/trocas" {
                        i .fa-solid.fa-right-left {} " Trocas"
                    }
                }
                li .nav-item {
                    a .nav-link href="/pedidos" {
                        i .fa-solid.fa-hand {} " Pedidos"
                    }
                }
            }


            ul .nav.flex-column {
                @match local_user {
                    LocalUser::Anonymous => {
                        li .nav-item {
                            a .nav-link href="/entrar" {
                                i .fa-solid.fa-lock {} " Entrar"
                            }
                        }
                        li .nav-item {
                            a .nav-link href="/registrar" {
                                i .fa-solid.fa-door-open {} " Criar conta"
                            }
                        }
                    }
                    LocalUser::Authenticated { id: _id, nickname: _nickname } => {
                        li .nav-item {
                            a .nav-link href="/minha-conta" {
                                i .fa-solid.fa-user {} " Minha conta"
                            }
                        }
                        li .nav-item {
                            a .nav-link href="/configurações" {
                                i .fa-solid.fa-gear {} " Configurações"
                            }
                        }
                        li .nav-item {
                            a .nav-link href="/sair" {
                                i .fa-solid.fa-lock-open {} " Sair"
                            }
                        }
                    }
                }
            }

            // footer stuff
            small .text-muted.mb-3 {
                "Compartilhe objetos com colegas e ajude a criar uma universidade com menos consumo."
            }
            div .row.g-2.justify-content-center {
                div .col.text-center.text-nowrap {
                    a .text-decoration-none href="/política-de-privacidade" {
                        small { "Política de privacidade" }
                    }
                }
                div .col.text-center.text-nowrap {
                    a .text-decoration-none href="/termos-de-uso" {
                        small { "Termos de uso" }
                    }
                }
                div .col.text-center.text-nowrap {
                    a .text-decoration-none href="/diretrizes-da-comunidade" {
                        small { "Diretrizes da Comunidade" }
                    }
                }
                div .col.text-center.text-nowrap {
                    a .text-decoration-none href="/sobre" {
                        small { "Sobre" }
                    }
                }
            }
            div .row {
                a .text-decoration-none.text-center href="https://github.com/wesley950/coisando-coisas" {
                    small {
                        i .fa-brands.fa-github {} " Código fonte"
                    }
                }
            }
        }
    }
}
