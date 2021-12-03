pub fn add_assett(m, list, dblist) {
    let assett_list: Vec<_> = list.into_iter()
        .filter(|x| x["object"] == "assett").map(|x| x["subject"].as_str().unwrap())
        .collect();
    // dbg!(m.value_of("startdate").unwrap());
    let name: &str = m.value_of("name").unwrap();
    // ensure assett does not already exist
    assert!(!assett_list.contains(&name), "Assett already exists");
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
        // println!(
        //     "made it to adding assett {}, {}, {:?}, {}, {}, {}, {}, {:?}",
        //     name, numb, startdate, min, med, max, appreciation, owners
        //     )
        let db_json = json!(dblist);
        println!("{}", db_json.to_string());
        // save db
        fs::write(path, db_json.to_string())
            .expect("Unable to save to database... ");
}
