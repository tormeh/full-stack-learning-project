#![feature(plugin)]

#![plugin(rocket_codegen)]

extern crate rocket;
extern crate postgres;
extern crate rocket_contrib;
extern crate uuid;
extern crate toml;

//#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

//#[cfg(test)] mod tests;

//use rocket_contrib::{Json, Value};
use rocket_contrib::{Json, UUID};
use rocket::State;
use rocket::response::status;
use rocket::response::status::{Created, BadRequest, Accepted};

use postgres::{Connection, TlsMode};
use uuid::Uuid;
use std::env;
use std::process;
use toml::Value;
use std::fs::File;
use std::io::prelude::*;


#[derive(Serialize)]
struct SchemalessTable {
    rows: Vec<SchemalessRow>,
}
#[derive(Serialize)]
struct SchemalessRow {
    id: Uuid,
    content: String,
}

#[derive(Deserialize)]
struct itemBody {
    content: String
}


struct Settings {
    db_url: String
}


fn get_connection(settings: &Settings) -> postgres::Connection 
{
    return Connection::connect(settings.db_url.to_string(), TlsMode::None).expect("Could not connect to postgres");
}

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[get("/hello/<name>/<age>")]
fn hello_name(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/list")]
fn list(settings: State<Settings>) -> Json<SchemalessTable>
{
    let conn = get_connection(settings.inner());
    let mut table = SchemalessTable {rows: Vec::new()};
    for row in &conn.query("SELECT * FROM test", &[]).unwrap() 
    {
        table.rows.push(SchemalessRow{id: row.get(0), content: row.get(1)})
        //println!("Found row {}, with {}", table.id table.content);
    };
    Json(table)
}

#[get("/item/<item>")]
fn item(item: UUID, settings: State<Settings>) -> Option<Json<SchemalessRow>>
{
    let conn = get_connection(settings.inner());
    let query = format!("SELECT * FROM test where id = '{}'", item.into_inner().hyphenated().to_string());
    let queryresult = &conn.query(&query, &[]).unwrap();
    if queryresult.is_empty()
    {
        None
    }
    else
    {
        let row = queryresult.get(0);
        let result = SchemalessRow{id: row.get(0), content: row.get(1)};
        Some(Json(result))
    }
    
}

#[delete("/item/<item>")]
fn item_delete(item: UUID, settings: State<Settings>) -> Result<Accepted<()>, BadRequest<()>>
{
    let conn = get_connection(settings.inner());
    let query = format!("DELETE FROM test where id = '{}'", item.into_inner().hyphenated().to_string());
    println!("Query: {}", query);
    let queryresult = conn.query(&query, &[]);

    match queryresult {
        Ok(o) => return Result::Ok(status::Accepted::<()>(None)),
        Err(e) => {println!("{}",e); return Result::Err(status::BadRequest::<()>(None))}
        
    };
    
}

#[post("/item", data = "<item_body>")]
fn item_post(item_body: Json<itemBody>, settings: State<Settings>) -> Result<Accepted<()>, BadRequest<()>>
{
    let conn = get_connection(settings.inner());
    let queryresult = conn.execute("INSERT INTO test (content) VALUES ($1::text)", &[&item_body.content]);

    match queryresult {
        Ok(o) => return Result::Ok(status::Accepted::<()>(None)),
        Err(e) => {println!("{}",e); return Result::Err(status::BadRequest::<()>(None))}
        
    };
    
}

#[put("/item/<item>", data = "<item_body>")]
fn item_put(item: UUID, item_body: Json<itemBody>, settings: State<Settings>) -> Result<Accepted<()>, BadRequest<()>>
{
    let conn = get_connection(settings.inner());
    let query = format!("UPDATE test set content = $1 WHERE id = '{}'", item.into_inner().hyphenated().to_string());
    let queryresult = conn.execute(&query, &[&item_body.content]);

    match queryresult {
        Ok(o) => return Result::Ok(status::Accepted::<()>(None)),
        Err(e) => {println!("{}",e); return Result::Err(status::BadRequest::<()>(None))}
        
    };
    
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file = match args.iter().find(|&x| x.starts_with("--config-file=")) 
    {
        Some(value) => value.trim_left_matches("--config-file="),
        None => {println!("No config file specified. Exiting."); process::exit(1)}
    };
    println!("Config file specified: {}", config_file);
    let mut file = File::open(config_file).expect("Could not open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Could not read config file");

    let settings_list = contents.parse::<Value>().expect("Error while parsing config file");
    let db_url = settings_list["db_url"].as_str().expect("No db_url specified in config file");
    println!("db_url: {}", db_url);
    let settings = Settings{db_url: db_url.to_string()};
    rocket::ignite().manage(settings).mount("/", routes![hello, hello_name, list, item, item_post, item_put, item_delete]).launch();
}
