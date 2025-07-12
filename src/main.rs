mod store;
use store::Store;

fn main() {
    let mut db = Store::new();
    db.set("name", "Medusa");

    if let Some(value) = db.get("name") {
        println!("Got: {}", value);
    }
}
