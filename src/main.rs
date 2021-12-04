#[macro_use]
extern crate clap;
use std::fs;
use std::path::{PathBuf};
use std::process;
use home;
use serde_json::{Value};
use serde_json::json;
use dialoguer::Confirm;
use numberkit::{is_number, as_f32, as_f32d};
// use chrono::{NaiveDate};

fn main() {
    use clap::App;
    
    let yml = load_yaml!("cli.yaml");
    let m = App::from(yml).get_matches();

    // declare important variables
    let home = home::home_dir().unwrap();
    // list of possible financial items
    const ITEMS: [(&str, &str); 5] = [
        ("Assetts", "assett"), ("Debts", "debt"), ("Income", "income"),
        ("Expenses", "expense"), ("Environment", "environment")
    ];    
    let path: PathBuf;
    if let Some(m) = m.value_of("database") {
        path = PathBuf::from(m);
    } else {
        path = home.join(".finances/database.json");
    };

    let data = fs::read_to_string(&path)
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
                let value: Vec<_> = list.into_iter()
                    .filter(|x| x["predicate"].eq("med") & x["subject"].eq(&assett["subject"]))
                    .collect();
                let numb: Vec<_> = list.into_iter()
                    .filter(|x| x["predicate"].eq("number") & x["subject"].eq(&assett["subject"]))
                    .collect();
                let freq: Vec<_> = list.into_iter()
                    .filter(|x| x["predicate"].eq("frequency") & x["subject"].eq(&assett["subject"]))
                    .collect();
                let v: f64;
                if numb.len() == 1 {
                    v = value[0]["object"].as_f64().unwrap() * numb[0]["object"].as_f64().unwrap(); 
                } else {
                    v = value[0]["object"].as_f64().unwrap() 
                };
                let f: String;
                if freq.len() == 1 {
                    f = freq[0]["object"].to_string(); 
                } else {
                    f = String::from("")
                };
                if owners.len() > 0 {
                    println!("{}, attached to: {:?}, ${} {}", assett["subject"], owners, v, f)
                } else {
                    println!("{}, ${} {}", assett["subject"], v, f)
                };
            }; 
          }
        }
    }

    if let Some(m) = m.subcommand_matches("delete") {
        if let Some(m) = m.value_of("name") {
            if Confirm::new().with_prompt(format!("Do you want to delete {}?", m))
                .interact().unwrap() {
                let subset: Vec<_> = list.into_iter().filter(
                    |x| x["object"] != m && x["subject"] != m
                    ).collect();
                let db_json = json!(subset);
                println!("{}", db_json.to_string());
                // save db
                fs::write(&path, db_json.to_string())
                    .expect("Unable to save to database... ");
            } else {
                println!("Delete avoided :)")
            }
        }
    }

    // set up add -- note that m is shadowing in code blocks
    if let Some(m) = m.subcommand_matches("add") {
        if let Some(m) = m.subcommand_matches("person") {
            let name: &str = m.value_of("name").unwrap();
            namecheck(&list, &name, true);
            let person = json!({
                "subject": String::from(name),
                "predicate": String::from("is"),
                "object":String::from("person")
            });
            dblist.push(person);
            savedb(&dblist, &path);
        }
        // ADD ASSETT LOGIC
        if let Some(m) = m.subcommand_matches("assett") {
            let name: &str = m.value_of("name").unwrap();
            namecheck(&list, &name, true);
            // ensure assett does not already exist
            let class = json!({
                "subject": String::from(name),
                "predicate": String::from("is"),
                "object": String::from("assett")
            });
            dblist.push(class);
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
            for owner in owners {
                println!("{:?}", owner);
                let own_obj = json!({
                    "subject": owner.0,
                    "predicate": owner.1,
                    "object": String::from(name)
                });
                dblist.push(own_obj);
            };
            savedb(&dblist, &path);
        }
        if let Some(m) = m.subcommand_matches("debt") {
            let name: &str = m.value_of("name").unwrap();
            namecheck(&list, &name, true);
            // ensure assett does not already exist
            let class = json!({
                "subject": String::from(name),
                "predicate": String::from("is"),
                "object": String::from("debt")
            });
            dblist.push(class);
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
            for owner in owners {
                println!("{:?}", owner);
                let own_obj = json!({
                    "subject": owner.0,
                    "predicate": owner.1,
                    "object": String::from(name)
                });
                dblist.push(own_obj);
            };
            savedb(&dblist, &path);
        }
        if let Some(m) = m.subcommand_matches("income") {
            let name: &str = m.value_of("name").unwrap();
            namecheck(&list, &name, true);
            // ensure assett does not already exist
            let class = json!({
                "subject": String::from(name),
                "predicate": String::from("is"),
                "object": String::from("income")
            });
            dblist.push(class);
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
            let frequency = json!({
                "subject": String::from(name),
                "predicate": String::from("frequency"),
                "object": m.value_of("frequency").unwrap().to_string()
            });
            dblist.push(frequency);

            let appreciation = json!({
                "subject": String::from(name),
                "predicate": String::from("appreciation"),
                "object": m.value_of("appreciation").unwrap().to_string().parse::<f32>().unwrap()
            });
            dblist.push(appreciation);
            let owners: Vec<_> = m.values_of("owners").unwrap().collect();
            let owners: Vec<_> = owners.into_iter().map(|x| ownsplit(x.to_string())).collect();
            let owners: Vec<_> = normalize(owners);
            for owner in owners {
                println!("{:?}", owner);
                let own_obj = json!({
                    "subject": owner.0,
                    "predicate": owner.1,
                    "object": String::from(name)
                });
                dblist.push(own_obj);
            };
            savedb(&dblist, &path);
        }
        if let Some(m) = m.subcommand_matches("expense") {
            let name: &str = m.value_of("name").unwrap();
            namecheck(&list, &name, true);
            // ensure assett does not already exist
            let class = json!({
                "subject": String::from(name),
                "predicate": String::from("is"),
                "object": String::from("expense")
            });
            dblist.push(class);
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
            let frequency = json!({
                "subject": String::from(name),
                "predicate": String::from("frequency"),
                "object": m.value_of("frequency").unwrap().to_string()
            });
            dblist.push(frequency);

            let appreciation = json!({
                "subject": String::from(name),
                "predicate": String::from("appreciation"),
                "object": m.value_of("appreciation").unwrap().to_string().parse::<f32>().unwrap()
            });
            dblist.push(appreciation);
            let owners: Vec<_> = m.values_of("owners").unwrap().collect();
            let owners: Vec<_> = owners.into_iter().map(|x| ownsplit(x.to_string())).collect();
            let owners: Vec<_> = normalize(owners);
            for owner in owners {
                println!("{:?}", owner);
                let own_obj = json!({
                    "subject": owner.0,
                    "predicate": owner.1,
                    "object": String::from(name)
                });
                dblist.push(own_obj);
            };
            savedb(&dblist, &path);
        }

    }

    // set up editing
    if let Some(m) = m.subcommand_matches("edit") {
        let subj = m.value_of("subject").unwrap();
        let pred = m.value_of("predicate").unwrap();
        let obj = m.value_of("object").unwrap();
        if is_number(pred) {
            let edit = json!({
            "subject": String::from(subj),
            "predicate": as_f32(pred).unwrap(),
            "object":String::from(obj)
            });
            namecheck(&list, &subj, false);
            namecheck(&list, &obj, false);
            dblist.retain(|x| !(x["subject"].eq(subj) & x["object"].eq(obj)));
            dblist.push(edit);
            savedb(&dblist, &path);
        } else {
            namecheck(&list, &subj, false);
            predcheck(&pred, &obj);
            let edit;
            if is_number(obj) {
                edit = json!({
                "subject": String::from(subj),
                "predicate": String::from(pred),
                "object": as_f32d(obj, 999f32)
            });
            } else {
                edit = json!({
                "subject": String::from(subj),
                "predicate": String::from(pred),
                "object": String::from(obj)
            });
            }
            dblist.retain(|x| !(x["subject"].eq(subj) & x["predicate"].eq(pred)));
            dblist.push(edit);
            savedb(&dblist, &path);
        }
    }
}

#[allow(dead_code)]
fn savedb(
    db: &Vec<serde_json::value::Value>,
    path: &PathBuf
    ) {
    let db_json = json!(db);
    println!("{}", db_json.to_string());
    // save db
    fs::write(path, db_json.to_string())
        .expect("Unable to save to database... ");
}

fn predcheck(
    pred: &str,
    obj: &str
    ) {
    // list of non-float predicates
    const PREDICATES: [&str; 12] = [
        "min", "max", "med", "frequency", "appreciation", "volatility", "ticker", "type", "is",
        "number", "startdate", "enddate"
    ];
    const ITEMS: [&str; 6] = [
        "person", "assett", "debt", "income", "expense", "environment"
    ];
    if !PREDICATES.contains(&pred){
        let msg = format!("{} does not exist", &pred);
        println!("{}", msg);
        process::exit(1);
    }
    if pred == "is" {
        if !ITEMS.contains(&obj){
            let msg = format!("{} not in {:?}", &obj, ITEMS);
            println!("{}", msg);
            process::exit(1);
        }
    }
}

fn namecheck(
    list: &Vec<serde_json::value::Value>, 
    name: &str,
    dir: bool
    ) {
    let all_names: Vec<_> = list.into_iter()
        .filter(|x| x["predicate"] == "is").map(|x| x["subject"].as_str().unwrap())
        .collect();
    if dir {
        if all_names.contains(&name){
            let msg = format!("{} already exists", &name);
            println!("{}", msg);
            process::exit(1);
            // assert!(!all_names.contains(&name));
        }
    } else {
        if !all_names.contains(&name){
            let msg = format!("{} does not exist", &name);
            println!("{}", msg);
            process::exit(1);
            //assert!(all_names.contains(&name));
        }
    }
}

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
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
