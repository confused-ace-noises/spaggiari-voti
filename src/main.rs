use testing::{Api, Credentials, Grade};
use std::collections::HashMap;
use std::io;
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
pub async fn main() {
    // ----- INIT -----
    bees::init(Duration::from_millis(700));
    dotenvy::dotenv().unwrap();

    let uid: String;
    let password: String;
    // load env variables from .env
    let mode = read_input::<usize, _>("Please input credentials sourcing mode (default: 1): \n 1: read from .env \n 2: read from input (stdin)", |n| *n == 1 || *n == 2);
    
    match mode {
        1 => {
            uid = dotenvy::var("UID").expect("FATAL: UID .env variable is missing");
            password = dotenvy::var("PASSWORD").expect("FATAL: PASSWORD .env variable is missing");
        },

        2 => {
            uid = read_input::<String, _>("Please input uid (email, student id, etc):", |_| true).trim().to_string();
            password = read_input::<String, _>("Please input password (this will be echoed because im lazy!!):", |_| true).trim().to_string();
        }

        _ => unreachable!()
    }

    let api = Api( Credentials { password, uid } );
    api.register().await;

    let grades: Vec<Grade> = api.grades().await;
    let grades: HashMap<String, Vec<f32>> = grades
        .into_iter()
        .map(|x| (x.subject_desc, x.decimal_value))
        .fold(HashMap::<String, Vec<f32>>::new(), |mut acc, (name, val)| {
            if let Some(val) = val {
                acc.entry(name)
                    .or_insert(vec![])
                    .push(val);
            }
            acc
        } );

    let averages = grades.iter().map(|(name, vals)| {
        let len = vals.len() as f32;
        (name.clone(), vals.iter().sum::<f32>()/len)
    }).collect::<HashMap<_, _>>();

    let general = grades.values().flatten().sum::<f32>()/grades.values().flatten().count() as f32;

    println!("---------- END LOGS ---------- \n \n general average: {}; \n grades: {:#?}; \n averages: {:#?}", general, grades, averages);
}

pub fn read_input<T, F>(call: &str, validate: F) -> T
where
    T: FromStr,
    F: Fn(&T) -> bool,
{
    loop {
        let mut buffer = String::new();
        println!("{}", call);
        io::stdin().read_line(&mut buffer).expect("FATAL: couldn't read from stdin");
    
        match buffer.parse::<T>() {
            Ok(parsed) => {
                if !validate(&parsed) {
                    println!("Please input a valid input:");
                    continue;
                } else {
                    return parsed
                }
            },
            Err(_) => println!("Please input a valid input:"),
        }
    }
}