use rocket::http;
use rocket::request;
use rocket::Outcome;
use rocket::State;
use rocket::config;
use r2d2;
use r2d2_redis::RedisConnectionManager;

const DEFAULT_ADDRESS: &'static str = "redis://localhost";
const CONFIG_KEY: &'static str = "redis_connection_address";

pub fn db_pool(app_config: &config::Config) -> Pool {
    let address = app_config
        .extras
        .get(CONFIG_KEY)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| DEFAULT_ADDRESS);
    let manager = RedisConnectionManager::new(address).expect("connection manager");
    let redis_config = Default::default();

    r2d2::Pool::new(redis_config, manager).expect("db pool")
}

// Rocket guard type: a wrapper around an r2d2 pool.
// In conjunction with
// `impl<'a, 'r> request::FromRequest<'a, 'r> for RedisConnection` (see below)
// it allows code like:
//   ```
//   #[get("/")]
//   pub fn show(db: RedisConnection) -> ...
//   ```
pub struct RedisConnection(pub r2d2::PooledConnection<RedisConnectionManager>);

// An alias to the type for a pool of redis connections.
type Pool = r2d2::Pool<RedisConnectionManager>;

// Retrieving a single connection from the managed database pool.
impl<'a, 'r> request::FromRequest<'a, 'r> for RedisConnection {
    type Error = ();

    fn from_request(request: &'a request::Request<'r>) -> request::Outcome<RedisConnection, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(RedisConnection(conn)),
            Err(_) => Outcome::Failure((http::Status::ServiceUnavailable, ())),
        }
    }
}
