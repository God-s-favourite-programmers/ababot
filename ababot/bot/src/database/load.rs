use std::error::Error;

use surrealdb::{Datastore, Session};

pub async fn load_all_data() -> Result<String, Box<dyn Error>> {
    let ds = Datastore::new("file://ababot_database").await?;
    let ses = Session::for_kv().with_ns("bot").with_db("ababot");
    let action = "SELECT * FROM entry ORDER BY when DESC;";
    let result = ds.execute(&action, &ses, None, false).await?;
    // let result = result.get(0).unwrap().output().unwrap().to_string();
    let mut all_data = String::new();
    for row in result {
        let row = row.output().unwrap();
        all_data.push_str(&row.to_string());
        println!("{}", row.to_string());
    }
    Ok(all_data)
}
