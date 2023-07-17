// use lazy_static::lazy_static;

#[derive(PartialEq, Clone)]
pub struct Policy {
    pub user_policy: Vec<String>,
    pub user_only: Vec<String>,
    pub courier_policy: Vec<String>,
    pub admin_policy: Vec<String>,
    pub analyst_policy: Vec<String>,
}

impl Policy {
    pub async fn new() -> Policy {
        Policy {
            user_policy: vec![
                "USER".to_owned(),
                "COURIER".to_owned(),
                "ADMIN".to_owned(),
                "ANALYST".to_owned(),
            ],
            user_only: vec!["USER".to_owned()],
            courier_policy: vec!["COURIER".to_owned(), "ADMIN".to_owned()],
            admin_policy: vec!["ADMIN".to_owned()],
            analyst_policy: vec!["ANALYST".to_owned(), "ADMIN".to_owned()],
        }
    }
}

// #[derive(PartialEq, Clone)]
// pub struct Policy<'a> {
//     pub user_policy: Vec<&'a str>,
//     pub user_only: Vec<&'a str>,
//     pub courier_policy: Vec<&'a str>,
//     pub admin_policy: Vec<&'a str>,
//     pub analyst_policy: Vec<&'a str>,
// }

// impl Policy<'_>{
//     pub async fn new() -> Policy<'static> {
//         Policy {
//             user_policy: vec![
//                 "USER",
//                 "COURIER",
//                 "ADMIN",
//                 "ANALYST"
//             ],
//             user_only: vec!["USER"],
//             courier_policy: vec!["COURIER", "ADMIN"],
//             admin_policy: vec!["ADMIN"],
//             analyst_policy: vec!["ANALYST", "ADMIN"],
//         }
//     }
// }

// lazy_static! {
//     pub static ref POLICY: Policy = Policy {
//         user_policy: vec![
//             "USER".to_owned(),
//             "COURIER".to_owned(),
//             "ADMIN".to_owned(),
//             "ANALYST".to_owned()
//         ],
//         user_only: vec!["USER".to_owned()],
//         courier_policy: vec!["COURIER".to_owned(), "ADMIN".to_owned()],
//         admin_policy: vec!["ADMIN".to_owned()],
//         analyst_policy: vec!["ANALYST".to_owned(), "ADMIN".to_owned()],
//     };
// }
