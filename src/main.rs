#[macro_use]
extern crate clap;
use std::fs;
use home;
use serde_json::{Value};
use serde_json::json;
use chrono::{NaiveDate};

fn main() {
    use clap::App;
    
    let yml = load_yaml!("cli.yaml");
    let m = App::from(yml).get_matches();

    // declare important variables
    let home = home::home_dir().unwrap();
    // list of possible financial items
    const ITEMS: [(&str, &str); 6] = [
        ("Assetts", "assett"), ("Debts", "debt"), ("Income", "income"),
        ("Expenses", "expense"), ("Payslips", "payslips"), ("Environment", "environment")
    ];
    // list of non-float predicates
    // const predicates: [&str; 9] = [
    //     "min", "max", "med", "frequency", "appreciation", "volatility", "ticker", "type", "is"
    // ];


    let data = fs::read_to_string(
        home.join(".finances/database.json"))
        .expect("Unable to get database... ");

    let db: Value = serde_json::from_str(&data).expect("JSON not formatted");
    let mut mutdb = db.clone();
    let list = db.as_array().unwrap();
    let dblist = mutdb.as_array_mut().unwrap();
    // println!("{:?}",list);
    let people = list.into_iter().filter(|x| x["object"] == "person").map(|x| &x["subject"]);

    // set up views
    if let Some(m) = m.subcommand_matches("view") {
        let class = m.value_of("class").unwrap();
        if class == "all" || class == "people" {
            println!("---- People ----");
            for entry in people {
                println!("{}", entry)
            };
        }
        for it in ITEMS {
            if class == "all" || class == it.0.to_lowercase() {
            println!("---- {} ----", it.0);
            let assetts = list.into_iter().filter(|x| x["object"] == it.1);
            for assett in assetts {
                let owners: Vec<_> = list.into_iter()
                .filter(|x| x["predicate"].is_number() & x["object"].eq(&assett["subject"]))
                .map(|x| (x["subject"].as_str().unwrap(), x["predicate"].as_f64().unwrap()))
                .collect();
                println!("{}, attached to: {:?}", assett["subject"], owners)
            }; 
          }
        }
    }

    // set up add -- note that m is shadowing in code blocks
    if let Some(m) = m.subcommand_matches("add") {
        // add assett
        if let Some(m) = m.subcommand_matches("assett") {
            // dbg!(m.value_of("startdate").unwrap());
            let name: &str = m.value_of("name").unwrap(); 
            let numb = json!({
                "subject": String::from(name),
                "predicate": String::from("number"),
                "object": m.value_of("number").unwrap().to_string().parse::<f32>().unwrap()
            });
            dblist.push(numb);
            let startdate = json!({
                "subject": String::from(name),
                "predicate": String::from("startdate"),
                "object": m.value_of("startdate").unwrap().to_string()
                // "object": NaiveDate::parse_from_str(
                // m.value_of("startdate").unwrap(),
                // "%Y-%m-%d"
                // ).unwrap()
            });
            dblist.push(startdate);
            let min = json!({
                "subject": String::from(name),
                "predicate": String::from("min"),
                "object": m.value_of("min").unwrap().to_string().parse::<f32>().unwrap()
            });
            dblist.push(min);
            let med = json!({
                "subject": String::from(name),
                "predicate": String::from("med"),
                "object": m.value_of("med").unwrap().to_string().parse::<f32>().unwrap()
            });
            dblist.push(med);
            let max = json!({
                "subject": String::from(name),
                "predicate": String::from("max"),
                "object": m.value_of("max").unwrap().to_string().parse::<f32>().unwrap()
            });
            dblist.push(max);
            let appreciation = json!({
                "subject": String::from(name),
                "predicate": String::from("appreciation"),
                "object": m.value_of("appreciation").unwrap().to_string().parse::<f32>().unwrap()
            });
            dblist.push(appreciation);
            let owners: Vec<_> = m.values_of("owners").unwrap().collect();
            let owners: Vec<_> = owners.into_iter().map(|x| ownsplit(x.to_string())).collect();
            let owners: Vec<_> = normalize(owners);
            println!("{:?}", owners);
            // println!(
            //     "made it to adding assett {}, {}, {:?}, {}, {}, {}, {}, {:?}",
            //     name, numb, startdate, min, med, max, appreciation, owners
            //     )
        }
    }
        
}

fn ownsplit(ownstring: String) -> (String, f32) {
    let mut splits = ownstring.split(":");
    let nam = splits.next().unwrap();
    let prop = splits.next().unwrap().to_string().parse::<f32>().unwrap();
    return (String::from(nam), prop)
}

fn normalize(ownvec: Vec<(String, f32)>) -> Vec<(String, f32)> {
    let total = ownvec.clone().into_iter().fold(0.0f32, |acc, x: (String, f32)| acc + x.1);
    if total < 1.0f32 {
        return ownvec.clone().into_iter().map(|x| (x.0, x.1 / total)).collect();
    } else {
        return ownvec;
    }
}
