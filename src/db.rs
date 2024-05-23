use mongodb::sync::{Client, options::ClientOptions};

pub fn conn() -> Result<(), Error> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(
        "mongodb://localhost:27017");

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // List the names of the databases in that deployment.
    for db_name in client.list_database_names(None, None) {
        println!("{:?}", db_name);
    }

    // Get a handle to a database.
    let db = client.database("gatekeeper_dev");

    // List the names of the collections in that database.
    for collection_name in db.list_collection_names(None) {
        println!("{:?}", collection_name);
    }
}
