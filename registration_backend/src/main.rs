#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate diesel;
extern crate dotenv;

use rocket_contrib::json::{Json, JsonValue};
use serde::{Serialize, Deserialize};
use diesel::{SqliteConnection, prelude::*};
use dotenv::dotenv;
use std::env;

// Data model
mod schema {
    table! {
        tasks {
            id -> Integer,
            title -> Text,
            description -> Text,
            completed -> Bool,
        }
    }
}
use schema::tasks;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "tasks"]
struct Task {
    id: Option<i32>,
    title: String,
    description: String,
    completed: bool,
}

#[get("/tasks")]
fn get_tasks(conn: DbConn) -> Json<Vec<Task>> {
    use schema::tasks::dsl::*;

    let result = tasks.load::<Task>(&*conn);
    match result {
        Ok(tasks) => Json(tasks),
        Err(_) => Json(vec![]),
    }
}

#[post("/tasks", format = "json", data = "<task>")]
fn create_task(task: Json<Task>, conn: DbConn) -> Json<Task> {
    use schema::tasks::dsl::*;

    let inserted_task = diesel::insert_into(tasks)
        .values(&*task)
        .execute(&*conn)
        .map_err(|_| ())
        .and_then(|_| {
            tasks.order(id.desc()).first::<Task>(&*conn).map_err(|_| ())
        });

    match inserted_task {
        Ok(task) => Json(task),
        Err(_) => Json(Task {
            id: None,
            title: "Error".to_string(),
            description: "Failed to create task".to_string(),
            completed: false,
        }),
    }
}

#[delete("/tasks/<id>")]
fn delete_task(id: i32, conn: DbConn) -> JsonValue {
    use schema::tasks::dsl::*;

    let delete_result = diesel::delete(tasks.filter(schema::tasks::id.eq(id)))
        .execute(&*conn)
        .map_err(|_| ());

    match delete_result {
        Ok(_) => json!({ "status": "success", "message": "Task deleted successfully" }),
        Err(_) => json!({ "status": "error", "message": "Task not found" }),
    }
}

// Rocket fairing to manage database connection pool
#[database("tasks_db")]
struct DbConn(diesel::SqliteConnection);

fn main() {
    dotenv().ok();
    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/", routes![get_tasks, create_task, delete_task])
        .launch();
}
