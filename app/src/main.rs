#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use owesome::app::*;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

    // Connection settings come from compose.yaml; the password from the secrets volume.
    let env = |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"));
    let password = std::fs::read_to_string(env("POSTGRES_PASSWORD_FILE"))
        .expect("cannot read Postgres password file");
    let db_options = PgConnectOptions::new()
        .host(&env("POSTGRES_HOST"))
        .username(&env("POSTGRES_USER"))
        .password(password.trim())
        .database(&env("POSTGRES_DB"));
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
        .await
        .expect("cannot connect to the database");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            // Runs for every request, so pages and server functions can use the pool.
            move || provide_context(pool.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    panic!("Somethings wrong, this project is intended for server side rendering");
}
