use surrealdb::{Datastore, Session};

pub async fn save(save_object: String) -> Result<String, Box<dyn std::error::Error>> {
    let ds = Datastore::new("file://ababot_database").await?;
    let ses = Session::for_kv().with_ns("bot").with_db("ababot");
    let action = format!(
        "CREATE entry SET when = time::now(), data = '{}';",
        save_object
    );
    ds.execute(&action, &ses, None, false).await?;
    Ok("".to_string())
}
