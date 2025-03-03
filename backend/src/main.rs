use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::env;

// Define the model that matches your table.
#[derive(Serialize, sqlx::FromRow)]
struct TableData {
    id: i32,
    name: String,
    email: String,
    role: String,
    created_at: DateTime<Utc>,
}

// Handler to fetch data from the database.
async fn get_table_data(pool: web::Data<sqlx::PgPool>) -> impl Responder {
    println!("Received request for table data.");

    let result = sqlx::query_as::<_, TableData>(
        "SELECT id, name, email, role, created_at FROM table_data"
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(data) => {
            println!("Successfully fetched {} rows.", data.len());
            for row in &data {
                println!(
                    "Row - id: {}, name: {}, email: {}, role: {}, created_at: {}",
                    row.id, row.name, row.email, row.role, row.created_at
                );
            }
            HttpResponse::Ok().json(data)
        }
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

    // Create a connection pool.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    println!("Starting server at http://127.0.0.1:9998");

    // Start the HTTP server with CORS enabled.
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()  // For production, restrict as needed.
                    .allow_any_method()
                    .allow_any_header()
            )
            .route("/table-data", web::get().to(get_table_data))
    })
    .bind("127.0.0.1:9998")?
    .run()
    .await
}
