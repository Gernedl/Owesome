use crate::auth::*;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::components::{Redirect, Route, Router, Routes, A};
use leptos_router::path;
use leptos_use::{use_clipboard, UseClipboardReturn};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

type UserRes = Resource<Result<Option<String>, ServerFnError>>;

#[component]
pub fn App() -> impl IntoView {
    // Needed for <Stylesheet> and <Title> to end up in the <head>.
    provide_meta_context();

    let login = ServerAction::<Login>::new();
    let logout = ServerAction::<Logout>::new();
    let register = ServerAction::<Register>::new();

    // Blocking => resolved on the server before HTML is sent,
    // so redirects become real 302s instead of client-side hops.
    let user: UserRes = Resource::new_blocking(
        move || (login.version().get(), logout.version().get()),
        |_| get_current_user(),
    );

    provide_context(user);
    provide_context(login);
    provide_context(logout);
    provide_context(register);

    view! {
        // Compiled from style/main.scss by cargo-leptos.
        <Stylesheet id="leptos" href="/pkg/owesome.css"/>
        <Title text="Owesome"/>

        <Router>
            <main>
                <Routes fallback=|| "Not found">
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/login") view=LoginPage/>
                    <Route path=path!("/register") view=RegisterPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let user = expect_context::<UserRes>();
    let logout = expect_context::<ServerAction<Logout>>();
    let get_code = ServerAction::<GetCode>::new();
    let UseClipboardReturn { copy, copied, .. } = use_clipboard();
    // `copy` may only be touched on the thread that created it, but on the server
    // <Suspense> can re-render on another thread. A StoredValue is a copyable handle,
    // and the function inside is only accessed when called (in the browser, on click).
    let copy = StoredValue::new(copy);

    view! {
        <Suspense>
            {move || user.get().map(|u| match u {
                Ok(Some(name)) => {
                    view! {
                    <h1>"Welcome, " {name}</h1>

                    // Step 1: ask the server for a code and show it.
                    <button on:click=move |_| { get_code.dispatch(GetCode {}); }>
                        "Create invite code"
                    </button>
                    // Step 2: a separate click copies it (browsers only allow
                    // clipboard writes directly in response to a click).
                    {move || get_code.value().get().map(|result| match result {
                        Ok(code) => {
                            let shown = code.clone();
                            view! {
                                <div class="invite">
                                    <code>{shown}</code>
                                    <button on:click=move |_| copy.with_value(|copy| copy(&code))>
                                        {move || if copied.get() { "Copied!" } else { "Copy" }}
                                    </button>
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <p class="error">"Could not create a code"</p>
                        }.into_any(),
                    })}

                    <ActionForm action=logout>
                        <button type="submit">"Log out"</button>
                    </ActionForm>
                }.into_any()
                }
                _ => view! { <Redirect path="/login"/> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    let user = expect_context::<UserRes>();
    let login = expect_context::<ServerAction<Login>>();

    view! {
        <Suspense>
            {move || user.get().map(|u| match u {
                // already logged in -> skip the login page
                Ok(Some(_)) => view! { <Redirect path="/"/> }.into_any(),
                _ => view! {
                    <h1>"Login"</h1>
                    <ActionForm action=login>
                        <input type="text" name="username" placeholder="Username"/>
                        <input type="password" name="password" placeholder="Password"/>
                        <button type="submit">"Log in"</button>
                    </ActionForm>
                    <Show when=move || login.value().get().is_some_and(|r| r.is_err())>
                        <p class="error">"Login failed"</p>
                    </Show>
                    <A href="/register">"No account yet? Register"</A>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn RegisterPage() -> impl IntoView {
    let user = expect_context::<UserRes>();
    let register = expect_context::<ServerAction<Register>>();

    view! {
        <Suspense>
            {move || user.get().map(|u| match u {
                // already logged in -> no need to register
                Ok(Some(_)) => view! { <Redirect path="/"/> }.into_any(),
                _ => view! {
                    <h1>"Register"</h1>
                    // The input names must match the parameters of the `register` server function.
                    <ActionForm action=register>
                        <input type="text" name="code" placeholder="Registration code"/>
                        <input type="text" name="username" placeholder="Username"/>
                        <input type="password" name="password" placeholder="Password"/>
                        <button type="submit">"Register"</button>
                    </ActionForm>
                    <Show when=move || register.value().get().is_some_and(|r| r.is_err())>
                        <p class="error">"Registration failed"</p>
                    </Show>
                    <A href="/login">"Already have an account? Log in"</A>
                }.into_any(),
            })}
        </Suspense>
    }
}
