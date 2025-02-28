use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::env;

// Define the model that matches your table
#[derive(Serialize, sqlx::FromRow)]
struct TableData {
    id: i32,
    name: String,
    email: String,
    role: String,
}

// Handler to fetch data from the database
async fn get_table_data(pool: web::Data<sqlx::PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, TableData>(
        "SELECT id, name, email, role FROM table_data"
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Error fetching data")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in your environment");

    // Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/table-data", web::get().to(get_table_data))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
