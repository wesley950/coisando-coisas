use std::io::Write;

use actix_identity::Identity;
use actix_web::{
    error::ErrorInternalServerError, get, post, web, HttpMessage, HttpRequest, HttpResponse,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use coisando_coisas::{
    schema::{confirmation_codes, sql_types::UserStatus, users},
    DbPool, LocalUser,
};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    query_dsl::methods::{FilterDsl, SelectDsl},
    serialize::{IsNull, ToSql},
    BoolExpressionMethods, Connection, ExpressionMethods, OptionalExtension, RunQueryDsl,
};
use maud::html;
use serde::Deserialize;
use uuid::Uuid;

use crate::pages::render_base;

#[derive(Deserialize)]
struct UserLoginForm {
    pub nickname: String,
    pub password: String,
}

// map db enum to rust enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = UserStatus)]
enum AccountStatus {
    PENDING,
    CONFIRMED,
    DISABLED,
}

impl ToSql<UserStatus, Pg> for AccountStatus {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match *self {
            AccountStatus::PENDING => out.write_all(b"PENDING")?,
            AccountStatus::CONFIRMED => out.write_all(b"CONFIRMED")?,
            AccountStatus::DISABLED => out.write_all(b"DISABLED")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<UserStatus, Pg> for AccountStatus {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"PENDING" => Ok(AccountStatus::PENDING),
            b"CONFIRMED" => Ok(AccountStatus::CONFIRMED),
            b"DISABLED" => Ok(AccountStatus::DISABLED),
            _ => Err("Unknown user status".into()),
        }
    }
}

#[post("/entrar")]
async fn login_user(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    details: web::Form<UserLoginForm>,
) -> Result<HttpResponse, actix_web::Error> {
    // get a connection from the pool
    let Ok(mut conn) = pool.get() else {
        return Err(ErrorInternalServerError(
            "Não foi possível conectar ao banco de dados",
        ));
    };

    // get user's hashed password
    let Ok(creds) = users::table
        .filter(
            users::nickname
                .eq(&details.nickname)
                .and(users::status.eq(AccountStatus::CONFIRMED)),
        )
        .select((users::id, users::hashed_password))
        .first::<(Uuid, String)>(&mut conn)
        .optional()
    else {
        return Err(ErrorInternalServerError(
            "Não foi possível verificar suas credenciais",
        ));
    };

    // check if user exists
    let Some((user_id, hashed_pass)) = creds else {
        // redirect, showing an error message
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/entrar?erro=credenciais"))
            .finish());
    };

    // verify password
    let argon2 = Argon2::default();
    let Ok(parsed_password_hash) = PasswordHash::new(&hashed_pass) else {
        return Err(ErrorInternalServerError(
            "Não foi possível verificar suas credenciais",
        ));
    };
    let Ok(_) = argon2.verify_password(details.password.as_bytes(), &parsed_password_hash) else {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/entrar?erro=credenciais"))
            .finish());
    };

    // log this user in
    let Ok(_) = Identity::login(&req.extensions(), user_id.simple().to_string()) else {
        return Err(ErrorInternalServerError(
            "Não foi possível criar uma sessão para você",
        ));
    };

    // success, redirect to account page
    return Ok(HttpResponse::Found()
        .append_header(("Location", "/minha-conta"))
        .finish());
}

#[derive(Deserialize)]
struct ErrorQuery {
    erro: Option<String>,
}

#[get("/entrar")]
async fn login_page(local_user: LocalUser, error: web::Query<ErrorQuery>) -> HttpResponse {
    let markup = render_base(
        html! {
            form .vstack.gap-3 method="post" action="/entrar" {
                h1 { "Entrar" }
                @if let Some(ref error) = error.erro {
                    div .alert.alert-danger role="alert" { (match error.as_str() {
                        "credenciais" => "Credenciais inválidas.",
                        _ => "Erro desconhecido."
                    }) }
                }
                input .form-control type="text" name="nickname" placeholder="Apelido";
                input .form-control type="password" name="password" placeholder="Senha";
                // TODO: add a captcha here
                button .btn.btn-primary type="submit" { "Enviar" }
            }
        },
        local_user,
    ); // not necessary to have user's nickname here
    HttpResponse::Ok().body(markup.into_string())
}

#[derive(Deserialize)]
struct UserRegisterForm {
    pub nickname: String,
    pub email: String,
    pub password: String,
}

// error enum for user registration
#[derive(Debug)]
enum UserRegisterError {
    InternalServerError,
    NicknameInUse,
    EmailInUse,
    PasswordTooShort,
    PasswordWeak,
    UnableToHashPassword,
    UnableToCreateUser,
    UnableToCreateConfirmationCode,
    UnableToSendEmail,
}

// implement this From<> so we can rollback the transaction and return a meaningful error for the user
impl From<UserRegisterError> for diesel::result::Error {
    fn from(_: UserRegisterError) -> Self {
        diesel::result::Error::RollbackTransaction
    }
}

// required by diesel, im not sure why
impl From<diesel::result::Error> for UserRegisterError {
    fn from(_: diesel::result::Error) -> Self {
        UserRegisterError::InternalServerError
    }
}

// function to send a confirmation email to the user
fn send_confirmation_email(_email: &str, _code: Uuid) -> Result<(), ()> {
    // TODO: in production, use a real email service
    // send an email to the user with the confirmation code
    // this is just a placeholder, so it always returns Ok and we manually verify the account
    Ok(())
}

#[post("/registrar")]
async fn register_new_user(
    pool: web::Data<DbPool>,
    details: web::Form<UserRegisterForm>,
) -> Result<HttpResponse, actix_web::Error> {
    // get a connection from the pool
    let Ok(mut conn) = pool.get() else {
        return Err(ErrorInternalServerError(
            "Não foi possível conectar ao banco de dados",
        ));
    };

    let transaction_result = conn.transaction::<(), UserRegisterError, _>(|conn| {
        // TODO: check if nickname is too short

        // TODO: check if nickname has only alphanumeric characters and underscores

        // TODO: do the same checks for changing nickname

        // check if nickname is already taken
        let Ok(nickname_in_use) = users::table
            .filter(users::nickname.eq(&details.nickname))
            .select(users::nickname)
            .first::<String>(conn)
            .optional()
        else {
            return Err(UserRegisterError::InternalServerError);
        };

        if nickname_in_use.is_some() {
            return Err(UserRegisterError::NicknameInUse);
        }

        // TODO: check email domain

        // check if email is already taken
        let Ok(email_in_use) = users::table
            .filter(users::email.eq(&details.email))
            .select(users::email)
            .first::<String>(conn)
            .optional()
        else {
            return Err(UserRegisterError::InternalServerError);
        };

        if email_in_use.is_some() {
            return Err(UserRegisterError::EmailInUse);
        }

        // check password strength
        // TODO: also do the same checks for changing password
        let requirements = [
            "abcdefghijklmnopqrstuvwxyz",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "0123456789",
            "!@#$%^&*()-_=+[]{}|;:,.<>/?",
        ];
        if details.password.len() < 8 {
            // redirect, showing an error message
            return Err(UserRegisterError::PasswordTooShort);
        } else if requirements
            .iter()
            .any(|req| !req.chars().any(|c| details.password.contains(c)))
        {
            // redirect, showing an error message
            return Err(UserRegisterError::PasswordWeak);
        }

        // hash this bitch!
        let salt = SaltString::generate(OsRng);
        let argon2 = Argon2::default();
        let Ok(hashed_pass) = argon2.hash_password(details.password.as_bytes(), &salt) else {
            return Err(UserRegisterError::UnableToHashPassword);
        };

        // insert new user
        let new_user_id = match diesel::insert_into(users::table)
            .values((
                users::id.eq(Uuid::new_v4()),
                users::nickname.eq(&details.nickname),
                users::email.eq(&details.email),
                users::hashed_password.eq(hashed_pass.to_string()),
                users::avatar_seed.eq(Uuid::new_v4()),
            ))
            .returning(users::id)
            .get_result::<Uuid>(conn)
        {
            Ok(user_id) => user_id,
            Err(err) => {
                println!("{:?}", err);
                return Err(UserRegisterError::UnableToCreateUser);
            }
        };

        // create confirmation code
        let confirmation_code = Uuid::new_v4();
        let Ok(_) = diesel::insert_into(confirmation_codes::table)
            .values((
                confirmation_codes::user_id.eq(new_user_id),
                confirmation_codes::code.eq(confirmation_code),
            ))
            .execute(conn)
        else {
            return Err(UserRegisterError::UnableToCreateConfirmationCode);
        };

        // try send to user via email
        let Ok(_) = send_confirmation_email(&details.email, confirmation_code) else {
            return Err(UserRegisterError::UnableToSendEmail);
        };

        Ok(())
    });

    match transaction_result {
        Ok(_) => (),
        Err(UserRegisterError::InternalServerError) => {
            return Err(ErrorInternalServerError(
                "Não foi possível criar sua conta devido a um erro interno.",
            ));
        }
        Err(UserRegisterError::NicknameInUse) => {
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/registrar?erro=apelido"))
                .finish());
        }
        Err(UserRegisterError::EmailInUse) => {
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/registrar?erro=email"))
                .finish());
        }
        Err(UserRegisterError::PasswordTooShort) => {
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/registrar?erro=senha-curta"))
                .finish());
        }
        Err(UserRegisterError::PasswordWeak) => {
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/registrar?erro=senha-fraca"))
                .finish());
        }
        Err(UserRegisterError::UnableToHashPassword) => {
            return Err(ErrorInternalServerError(
                "Não foi possível criptografar a sua senha devido a um erro interno.",
            ));
        }
        Err(UserRegisterError::UnableToCreateUser) => {
            return Err(ErrorInternalServerError(
                "Não foi possível salvar as suas informações devido a um erro interno.",
            ));
        }
        Err(UserRegisterError::UnableToCreateConfirmationCode) => {
            return Err(ErrorInternalServerError(
                "Não foi possível criar o código de confirmação devido a um erro interno.",
            ));
        }
        Err(UserRegisterError::UnableToSendEmail) => {
            return Err(ErrorInternalServerError(
                "Não foi possível enviar o email de confirmação devido a um erro interno.",
            ));
        }
    };

    // success, so redirect the user to the confirmation page (/confirmação)
    Ok(HttpResponse::Found()
        .append_header(("Location", "/confirmação"))
        .finish())
}

#[get("/registrar")]
async fn register_page(local_user: LocalUser, error: web::Query<ErrorQuery>) -> HttpResponse {
    if let LocalUser::Authenticated { .. } = local_user {
        return HttpResponse::Found()
            .append_header(("Location", "/minha-conta"))
            .finish();
    }

    let markup = render_base(
        html! {
            form .vstack.gap-3 method="post" action="/registrar" {
                h1 { "Criar conta" }
                @if let Some(ref error) = error.erro {
                    div .alert.alert-danger role="alert" { (match error.as_str() {
                        "apelido" => "O apelido já está em uso.",
                        "email" => "O email já está em uso.",
                        "senha-curta" => "A senha é muito curta.",
                        "senha-fraca" => "A senha é muito fraca.",
                        _ => "Erro desconhecido."
                    }) }
                }
                input .form-control type="text" name="nickname" placeholder="Apelido";
                input .form-control type="email" name="email" placeholder="Email";
                input .form-control type="password" name="password" placeholder="Senha";
                small { "Sua senha precisa ter pelo menos 8 caracteres, uma letra maiúscula e uma minúscula, um dígito e um dos seguintes símbolos: !@#$%^&*()-_=+[]{}|;:,.<>/?" }
                .form-check {
                    input .form-check-input type="checkbox" id="terms" name="terms" required;
                    label .form-check-label for="terms" {
                        "Ao me cadastrar, concordo com os "
                        a href="/termos-de-uso" { "termos de uso" } ", a "
                        a href="/política-de-privacidade" { "política de privacidade" } " e as "
                        a href="/diretrizes-da-comunidade" { "diretrizes da comunidade" } "."
                    }
                }
                // TODO: add a captcha here
                button .btn.btn-primary type="submit" { "Enviar" }
            }
        },
        local_user,
    ); // also not necessary to have user's nickname here
    HttpResponse::Ok().body(markup.into_string())
}

#[derive(Deserialize)]
struct VerificationInfo {
    code: Uuid,
}

// enum for user verification
enum UserVerificationError {
    InternalServerError,
    CodeInvalid,
    UnableToConfirmAccount,
    UnableToCreateSession,
}

// implement this From<> so we can rollback the transaction and return a meaningful error for the user
impl From<UserVerificationError> for diesel::result::Error {
    fn from(_: UserVerificationError) -> Self {
        diesel::result::Error::RollbackTransaction
    }
}

// required by diesel, im not sure why
impl From<diesel::result::Error> for UserVerificationError {
    fn from(_: diesel::result::Error) -> Self {
        UserVerificationError::InternalServerError
    }
}

/// GET /confirmar-conta?code=...
/// checks if the code in the database and confirm the user's account
#[get("/confirmar-conta")]
async fn confirm_account(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    details: web::Query<VerificationInfo>,
) -> Result<HttpResponse, actix_web::Error> {
    // get a connection from the pool
    let Ok(mut conn) = pool.get() else {
        return Err(ErrorInternalServerError(
            "Não foi possível conectar ao banco de dados",
        ));
    };

    // wrap everything in a transaction
    let transaction_result = conn.transaction::<(), UserVerificationError, _>(|conn| {
        // find the confirmation code in the database
        let Ok(user_id) = confirmation_codes::table
            .filter(confirmation_codes::code.eq(&details.code))
            .select(confirmation_codes::user_id)
            .first::<Uuid>(conn)
            .optional()
        else {
            return Err(UserVerificationError::InternalServerError);
        };

        let Some(user_id) = user_id else {
            return Err(UserVerificationError::CodeInvalid);
        };

        // we used the code, so delete it
        let Ok(_) = diesel::delete(confirmation_codes::table)
            .filter(confirmation_codes::code.eq(&details.code))
            .execute(conn)
        else {
            return Err(UserVerificationError::UnableToConfirmAccount);
        };

        // set the account status to confirmed
        let Ok(_) = diesel::update(users::table.filter(users::id.eq(&user_id)))
            .set(users::status.eq(AccountStatus::CONFIRMED))
            .execute(conn)
        else {
            return Err(UserVerificationError::UnableToConfirmAccount);
        };

        // lastly, log the user in
        let Ok(_) = Identity::login(&req.extensions(), user_id.simple().to_string()) else {
            return Err(UserVerificationError::UnableToCreateSession);
        };

        Ok(())
    });

    match transaction_result {
        Ok(_) => (),
        Err(UserVerificationError::InternalServerError) => {
            return Err(ErrorInternalServerError(
                "Não foi possível confirmar a sua conta devido a um erro interno.",
            ));
        }
        Err(UserVerificationError::CodeInvalid) => {
            return Ok(HttpResponse::BadRequest()
                .append_header(("Location", "/?erro=codigo-invalido"))
                .finish());
        }
        Err(UserVerificationError::UnableToConfirmAccount) => {
            return Err(ErrorInternalServerError(
                "Não foi possível confirmar a sua conta devido a um erro interno.",
            ));
        }
        Err(UserVerificationError::UnableToCreateSession) => {
            return Err(ErrorInternalServerError(
                "Não foi possível criar uma sessão para você",
            ));
        }
    }

    // success, redirect to account page
    return Ok(HttpResponse::Found()
        .append_header(("Location", "/minha-conta"))
        .finish());
}

#[get("/confirmação")]
async fn confirmation_page(local_user: LocalUser) -> HttpResponse {
    if let LocalUser::Authenticated { .. } = local_user {
        return HttpResponse::Found()
            .append_header(("Location", "/minha-conta"))
            .finish();
    }

    let markup = render_base(
        html! {
            h1 { "Verificação de email" }
            p { "Enviamos um email para você. Por favor, verifique sua caixa de entrada." }
        },
        local_user, // not strictly necessary to have user's nickname here
    );
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/minha-conta")]
async fn account_page(local_user: LocalUser) -> Result<HttpResponse, actix_web::Error> {
    // TODO: add part where user can manage their posts

    let (_user_id, nickname, avatar_seed) = match &local_user {
        LocalUser::Authenticated {
            id,
            nickname,
            avatar_seed,
        } => (id, nickname, avatar_seed),
        LocalUser::Anonymous => {
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/entrar"))
                .finish());
        }
    };

    let created_at = "TODO: get user's creation date and other info";
    let markup = render_base(
        html! {
            h1 { (format!("Perfil de {}", nickname)) }
            img .rounded-circle src=(format!("https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy", avatar_seed)) width="128" height="128" alt="avatar";
            p #createdAt { (format!("Conta criada em {}", created_at)) }
        },
        local_user,
    );

    Ok(HttpResponse::Ok().body(markup.into_string()))
}

#[get("/sair")]
async fn logout_user(id: Option<Identity>) -> HttpResponse {
    if let Some(id) = id {
        id.logout();
    }

    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

#[post("/settings/avatar")]
async fn generate_avatar(
    local_user: LocalUser,
    pool: web::Data<DbPool>,
) -> actix_web::Result<HttpResponse> {
    if let LocalUser::Anonymous = local_user {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/entrar"))
            .finish());
    } else if let LocalUser::Authenticated { id, .. } = local_user {
        // get a connection from the pool
        let Ok(mut conn) = pool.get() else {
            return Err(ErrorInternalServerError(
                "Não foi possível conectar ao banco de dados",
            ));
        };

        // generate a new avatar seed
        let new_avatar_seed = Uuid::new_v4();

        // update the user's avatar seed
        let Ok(_) = diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::avatar_seed.eq(new_avatar_seed))
            .execute(&mut conn)
        else {
            return Err(ErrorInternalServerError(
                "Não foi possível gerar um novo avatar",
            ));
        };
    };

    Ok(HttpResponse::Found()
        .append_header(("Location", "/configurações"))
        .finish())
}

#[derive(Deserialize)]
struct NewNicknameForm {
    nickname: String,
}

#[post("/settings/nickname")]
async fn change_nickname(
    local_user: LocalUser,
    pool: web::Data<DbPool>,
    new_nickname: web::Form<NewNicknameForm>,
) -> actix_web::Result<HttpResponse> {
    let new_nickname = new_nickname.into_inner();
    if let LocalUser::Authenticated { id, .. } = local_user {
        // get a connection from the pool
        let Ok(mut conn) = pool.get() else {
            return Err(ErrorInternalServerError(
                "Não foi possível conectar ao banco de dados",
            ));
        };

        // check if the new nickname is already in use
        let Ok(nickname_in_use) = users::table
            .filter(users::nickname.eq(&new_nickname.nickname))
            .select(users::nickname)
            .first::<String>(&mut conn)
            .optional()
        else {
            return Err(ErrorInternalServerError(
                "Não foi possível verificar o novo apelido",
            ));
        };
        if let Some(_) = nickname_in_use {
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/configurações?erro=apelido-em-uso"))
                .finish());
        }

        // update the user's nickname
        let Ok(_) = diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::nickname.eq(&new_nickname.nickname))
            .execute(&mut conn)
        else {
            return Err(ErrorInternalServerError(
                "Não foi possível alterar o seu apelido",
            ));
        };
    } else {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/entrar"))
            .finish());
    };

    // redirect to the settings page
    Ok(HttpResponse::Found()
        .append_header(("Location", "/configurações"))
        .finish())
}

#[derive(Deserialize)]
struct NewPasswordForm {
    password: String,
}

#[post("/settings/password")]
async fn change_password(
    local_user: LocalUser,
    pool: web::Data<DbPool>,
    new_password: web::Form<NewPasswordForm>,
) -> actix_web::Result<HttpResponse> {
    let new_password = new_password.into_inner();
    if let LocalUser::Authenticated { id, .. } = local_user {
        // get a connection from the pool
        let Ok(mut conn) = pool.get() else {
            return Err(ErrorInternalServerError(
                "Não foi possível conectar ao banco de dados",
            ));
        };

        // check password strength
        // TODO: move this to a function so the register page can use it too
        let requirements = [
            "abcdefghijklmnopqrstuvwxyz",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "0123456789",
            "!@#$%^&*()-_=+[]{}|;:,.<>/?",
        ];
        if new_password.password.len() < 8 {
            // redirect, showing an error message
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/configurações?erro=senha-curta"))
                .finish());
        } else if requirements
            .iter()
            .any(|req| !req.chars().any(|c| new_password.password.contains(c)))
        {
            // redirect, showing an error message
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/configurações?erro=senha-fraca"))
                .finish());
        }

        // hash the password
        let salt = SaltString::generate(OsRng);
        let argon2 = Argon2::default();
        let Ok(hashed_pass) = argon2.hash_password(new_password.password.as_bytes(), &salt) else {
            return Err(ErrorInternalServerError(
                "Não foi possível criptografar a sua senha",
            ));
        };

        // update the user's password
        let Ok(_) = diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::hashed_password.eq(hashed_pass.to_string()))
            .execute(&mut conn)
        else {
            return Err(ErrorInternalServerError(
                "Não foi possível alterar a sua senha",
            ));
        };
    } else {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/entrar"))
            .finish());
    };

    // redirect to the settings page
    Ok(HttpResponse::Found()
        .append_header(("Location", "/configurações"))
        .finish())
}

#[get("/configurações")]
async fn settings_page(local_user: LocalUser, error: web::Query<ErrorQuery>) -> HttpResponse {
    if let LocalUser::Anonymous = local_user {
        return HttpResponse::Found()
            .append_header(("Location", "/entrar"))
            .finish();
    }

    let markup = render_base(
        html! {
            h1 { "Configurações" }
            p { "Aqui você pode alterar suas informações." }

            @if let Some(ref error) = error.erro {
                div .alert.alert-danger role="alert" { (match error.as_str() {
                    "apelido-em-uso" => "O apelido já está em uso.",
                    "senha-curta" => "A senha é muito curta.",
                    "senha-fraca" => "A senha é muito fraca.",
                    _ => "Erro desconhecido."
                }) }
            }

            // button to generate a new avatar
            form .vstack.gap-3 method="post" action="/settings/avatar" {
                h2 { "Gerar novo avatar" }
                // preview of the new avatar
                @match &local_user {
                    LocalUser::Authenticated {avatar_seed, ..} => {
                        img src=(format!("https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy", avatar_seed)) class="rounded-circle" width="128" height="128" alt="avatar";
                    },
                    _ => {}
                }
                button .btn.btn-primary type="submit" { "Gerar" }
            }

            // change nickname
            form .vstack.gap-3 method="post" action="/settings/nickname" {
                h2 { "Alterar apelido" }

                @match &local_user {
                    LocalUser::Authenticated { nickname, .. } => {
                        p { (format!("Seu apelido atual é {}", nickname)) }
                    },
                    _ => {}
                };

                input .form-control type="text" name="nickname" placeholder="Novo apelido";
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
        },
        local_user,
    );
    HttpResponse::Ok().body(markup.into_string())
}

#[post("/settings/delete")]
async fn delete_account(
    pool: web::Data<DbPool>,
    local_user: LocalUser,
    id: Option<Identity>,
) -> actix_web::Result<HttpResponse> {
    // log user out
    if let Some(id) = id {
        id.logout();
    }

    if let LocalUser::Authenticated { id, .. } = local_user {
        // get a connection from the pool
        let Ok(mut conn) = pool.get() else {
            return Err(ErrorInternalServerError(
                "Não foi possível conectar ao banco de dados",
            ));
        };

        // delete user's account
        let Ok(_) = diesel::delete(users::table.filter(users::id.eq(id))).execute(&mut conn) else {
            return Err(ErrorInternalServerError(
                "Não foi possível deletar a sua conta",
            ));
        };
    };

    Ok(HttpResponse::Found()
        .append_header(("Location", "/conta-deletada"))
        .finish())
}

#[get("/conta-deletada")]
async fn deletion_confirmation_page(local_user: LocalUser) -> HttpResponse {
    if let LocalUser::Authenticated { .. } = local_user {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let markup = render_base(
        html! {
            h1 { "Conta deletada" }
            p { "Sua conta foi deletada com sucesso." }
        },
        local_user,
    );
    HttpResponse::Ok().body(markup.into_string())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login_user)
        .service(login_page)
        .service(register_new_user)
        .service(register_page)
        .service(confirm_account)
        .service(confirmation_page)
        .service(account_page)
        .service(logout_user)
        .service(generate_avatar)
        .service(change_nickname)
        .service(change_password)
        .service(settings_page)
        .service(delete_account)
        .service(deletion_confirmation_page);
}
