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

pub fn render_index() -> maud::Markup {
    let mock_items = vec![
        Item {
            title: "Livro de Matemática".to_string(),
            description: "Livro de matemática do ensino médio".to_string(),
            images: vec!["https://placehold.co/1280x1024".to_string()],
            user: User {
                username: "joao".to_string(),
                avatar_url: format!("https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy", "joao"),
            },
        },
        Item {
            title: "Cadeira de escritório".to_string(),
            description: "Cadeira de escritório em bom estado".to_string(),
            images: vec!["https://placehold.co/1024x1280".to_string()],
            user: User {
                username: "maria".to_string(),
                avatar_url: format!("https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy", "maria"),
            },
        },
        Item {
            title: "Notebook".to_string(),
            description: "Notebook com 8GB de RAM e 256GB de SSD".to_string(),
            images: vec!["https://placehold.co/1280x1024".to_string()],
            user: User {
                username: "joao".to_string(),
                avatar_url: format!("https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy", "joao"),
            },
        }
    ];

    render_base(html! {
        // hero
        h1 .text-center { "Bem-vindo ao Coisando Coisas!" }
        p .lead.text-center { "Onde estudantes compartilham, trocam e salvam o planeta. 😃" }

        // search form
        div .form-floating.mb-3 {
            input type="text" class="form-control" id="search" placeholder="";
            label for="search" { "Do que você precisa?" }
        }

        // results
        div .d-flex.flex-row.flex-wrap.gap-3 {
            @for item in mock_items {
                div .card.card-body style="width: 20rem;" {
                    div .vstack.gap-3 {
                        // todo: image carousel
                        div .img-fluid style="height: 350px;" {
                            img src=(item.images[0]) class="object-fit-contain w-100 h-100 bg-dark" alt=(item.title);
                        }

                        // details
                        h2 { (item.title) }
                        p { (item.description) }
                        // TODO: put link to profile page
                        p {
                            "Anunciado por "
                            a href="#" {
                                img src=(item.user.avatar_url) class="rounded-circle" width="32" height="32" alt=(item.user.username);
                                (item.user.username)
                            }
                        }
                    }
                }
            }
        }

        // pagination maybe?
    })
}
