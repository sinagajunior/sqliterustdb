use std::io;
use futures::future::err;

use ntex::web::{self,error,middleware,App,Error,HttpResponse};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use log::error;


async fn index(path: web::types::Path<String>,
               db: web::types::State<Pool<SqliteConnectionManager>>,
)-> Result<HttpResponse,Error>{
    let db = db.get_ref().clone();
    let res = web::block(move ||{
        let conn = db.get().unwrap();
        let uuid = format!("{}",uuid::Uuid::new_v4());
        conn.execute(
            "INSERT INTO users(id,name) valuse ($1,$2)",
            &[&uuid,&path.into_inner()],
        ).unwrap();

        conn.query_row("SELECT name FROM users where id=$1", &[&uuid],|row|{
            row.get::<_, String>(0)
        })
    })
        .await
        .map(|user|HttpResponse::Ok().json(&user))
        .map_err(error::ErrorInternalServerError)?;
        Ok(res)
}






#[ntex::main]
async fn main()->io::Result<()> {
    std::env::set_var("RUST_LOG","ntex=debug");
    env_logger::init();

    ///r2d2 pool
    let manager = SqliteConnectionManager::file("test.db");
    let pool = r2d2::Pool::new(manager).unwrap();

    //start http server
    web::server(move||{
        App::new()
            .state(pool.clone())
            .wrap(middleware::Logger::default())

            .route("/{name}",web::get().to(index))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}


