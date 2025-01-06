use actix_web::{get, web, HttpResponse};
use maud::html;

use crate::pages::render_base;

#[get("/entrar")]
async fn login_page() -> HttpResponse {
    let markup = render_base(html! {
        form .vstack.gap-3 method="post" action="/login" {
            h1 { "Entrar" }
            input .form-control type="text" name="username" placeholder="Apelido";
            input .form-control type="password" name="password" placeholder="Senha";
            button .btn.btn-primary type="submit" { "Enviar" }
        }
    });
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/registrar")]
async fn register_page() -> HttpResponse {
    let markup = render_base(html! {
        form .vstack.gap-3 method="post" action="/register" {
            h1 { "Criar conta" }
            input .form-control type="text" name="username" placeholder="Apelido";
            input .form-control type="email" name="email" placeholder="Email";
            input .form-control type="password" name="password" placeholder="Senha";
            button .btn.btn-primary type="submit" { "Enviar" }
        }
    });
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/confirmação")]
async fn verification_page() -> HttpResponse {
    let markup = render_base(html! {
        h1 { "Verificação de email" }
        p { "Enviamos um email para você. Por favor, verifique sua caixa de entrada." }
    });
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/minha-conta")]
async fn account_page() -> HttpResponse {
    let markup = render_base(html! {
        h1 { "Minha conta" }
        p { "Aqui você pode ver suas informações." }
    });
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/configurações")]
async fn settings_page() -> HttpResponse {
    let markup = render_base(html! {
        h1 { "Configurações" }
        p { "Aqui você pode alterar suas informações." }

        // button to generate a new avatar
        form .vstack.gap-3 method="post" action="/settings/avatar" {
            h2 { "Gerar novo avatar" }
            // preview of the new avatar
            img src="https://api.dicebear.com/9.x/dylan/svg?seed=maria&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy" class="rounded-circle" width="128" height="128" alt="avatar";
            button .btn.btn-primary type="submit" { "Gerar" }
        }

        // change nickname
        form .vstack.gap-3 method="post" action="/settings/nickname" {
            h2 { "Alterar apelido" }
            input .form-control type="text" name="username" placeholder="Novo apelido";
            button .btn.btn-primary type="submit" { "Enviar" }
        }

        // change email
        form .vstack.gap-3 method="post" action="/settings/email" {
            h2 { "Alterar email" }
            input .form-control type="email" name="email" placeholder="Novo email";
            button .btn.btn-primary type="submit" { "Enviar" }
        }

        // change password
        form .vstack.gap-3 method="post" action="/settings/password" {
            h2 { "Alterar senha" }
            input .form-control type="password" name="password" placeholder="Nova senha";
            button .btn.btn-primary type="submit" { "Enviar" }
        }

        // button to delete account
        form .vstack.gap-3 method="post" action="/settings/delete" {
            h2 { "Deletar conta" }
            p { "Tem certeza que deseja deletar sua conta? Esta ação é irreversível." }
            button .btn.btn-danger type="submit" { "Deletar conta" }
        }
    });
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/conta-deletada")]
async fn deletion_confirmation_page() -> HttpResponse {
    let markup = render_base(html! {
        h1 { "Conta deletada" }
        p { "Sua conta foi deletada com sucesso." }
    });
    HttpResponse::Ok().body(markup.into_string())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login_page)
        .service(register_page)
        .service(verification_page)
        .service(account_page)
        .service(settings_page)
        .service(deletion_confirmation_page);
}
