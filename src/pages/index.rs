use actix_web::{get, web, HttpResponse};
use maud::html;

use super::render_base;

struct User {
    username: String,
    avatar_url: String,
}

struct Item {
    title: String,
    description: String,
    images: Vec<String>,
    user: User,
}

#[get("/")]
async fn render_index() -> HttpResponse {
    let mock_items = vec![
        Item {
            title: "Livro de Matemática".to_string(),
            description: "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.\nContrary to popular belief, Lorem Ipsum is not simply random text. It has roots in a piece of classical Latin literature from 45 BC, making it over 2000 years old. Richard McClintock, a Latin professor at Hampden-Sydney College in Virginia, looked up one of the more obscure Latin words, consectetur, from a Lorem Ipsum passage, and going through the cites of the word in classical literature, discovered the undoubtable source. Lorem Ipsum comes from sections 1.10.32 and 1.10.33 of \"de Finibus Bonorum et Malorum\" (The Extremes of Good and Evil) by Cicero, written in 45 BC. This book is a treatise on the theory of ethics, very popular during the Renaissance. The first line of Lorem Ipsum, \"Lorem ipsum dolor sit amet..\", comes from a line in section 1.10.32.".to_string(),
            images: vec!["https://placehold.co/1280x1024".to_string()],
            user: User {
                username: "joao".to_string(),
                avatar_url: format!("https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy", "joao"),
            },
        },
        Item {
            title: "Cadeira de escritório".to_string(),
            description: "The standard chunk of Lorem Ipsum used since the 1500s is reproduced below for those interested. Sections 1.10.32 and 1.10.33 from \"de Finibus Bonorum et Malorum\" by Cicero are also reproduced in their exact original form, accompanied by English versions from the 1914 translation by H. Rackham.".to_string(),
            images: vec!["https://placehold.co/1024x1280".to_string()],
            user: User {
                username: "maria".to_string(),
                avatar_url: format!("https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy", "maria"),
            },
        },
        Item {
            title: "Notebook".to_string(),
            description: "There are many variations of passages of Lorem Ipsum available, but the majority have suffered alteration in some form, by injected humour, or randomised words which don't look even slightly believable. If you are going to use a passage of Lorem Ipsum, you need to be sure there isn't anything embarrassing hidden in the middle of text. All the Lorem Ipsum generators on the Internet tend to repeat predefined chunks as necessary, making this the first true generator on the Internet. It uses a dictionary of over 200 Latin words, combined with a handful of model sentence structures, to generate Lorem Ipsum which looks reasonable. The generated Lorem Ipsum is therefore always free from repetition, injected humour, or non-characteristic words etc.\nIt is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like).\nDonate: If you use this site regularly and would like to help keep the site on the Internet, please consider donating a small sum to help pay for the hosting and bandwidth bill. There is no minimum donation, any sum is appreciated - click here to donate using PayPal. Thank you for your support. Donate bitcoin: 16UQLq1HZ3CNwhvgrarV6pMoA2CDjb4tyF".to_string(),
            images: vec!["https://placehold.co/1280x1024".to_string()],
            user: User {
                username: "joao".to_string(),
                avatar_url: format!("https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy", "joao"),
            },
        }
    ];

    let markup = render_base(
        html! {
            // hero
            h1 .text-center { "Bem-vindo ao Coisando Coisas!" }
            p .lead.text-center { "Onde estudantes compartilham, trocam e salvam o planeta. 😃" }

            // search form
            div .form-floating.mb-3 {
                input type="text" class="form-control" id="search" placeholder="";
                label .text-muted for="search" { i .bi.bi-binoculars-fill {} " Do que você precisa?" }
            }

            // results
            div .d-flex.flex-row.flex-wrap.gap-3.justify-content-center {
                @for item in mock_items {
                    div .card.card-body.border-1.shadow style="width: 16em;" {
                        div .vstack.gap-2 {
                            // todo: image carousel
                            div .img-fluid style="height: 350px;" {
                                img src=(item.images[0]) class="object-fit-contain w-100 h-100 bg-dark" alt=(item.title);
                            }

                            // details
                            h2 { (item.title) }
                            // TODO: put link to profile page
                            p {
                                "Anunciado por "
                                a href="#" {
                                    img src=(item.user.avatar_url) class="rounded-circle" width="32" height="32" alt=(item.user.username);
                                    (item.user.username)
                                }
                            }
                            p .d-block.text-truncate.text-wrap style="height: 8em" { (item.description) }
                            // TODO: link to item page
                            a href="#" { "Detalhes" }
                        }
                    }
                }
            }

            // pagination maybe?
        },
        None,
    ); // TODO: replace with actual user

    HttpResponse::Ok().body(markup.into_string())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(render_index);
}
