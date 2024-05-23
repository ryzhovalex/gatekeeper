#![allow(unreachable_code)]
#[macro_use]
extern crate rouille;

fn main() {
    rouille::start_server("0.0.0.0:3000", move |request| {
        router!(request,
            (GET) (/) => {
                // If the request's URL is `/`, we jump here.
                // This block builds a `Response` object that redirects to the `/hello/world`.
                rouille::Response::redirect_302("/hello/world")
            },

            (GET) (/hello/world) => {
                // If the request's URL is `/hello/world`, we jump here.
                println!("hello world");

                // Builds a `Response` object that contains the "hello world" text.
                rouille::Response::text("hello world")
            },

            (GET) (/panic) => {
                // If the request's URL is `/panic`, we jump here.
                //
                // This block panics. Fortunately rouille will automatically catch the panic and
                // send back a 500 error message to the client. This prevents the server from
                // closing unexpectedly.
                panic!("Oops!")
            },

            (GET) (/{id: u32}) => {
                // If the request's URL is for example `/5`, we jump here.
                //
                // The `router!` macro will attempt to parse the identifier (eg. `5`) as a `u32`. If
                // the parsing fails (for example if the URL is `/hello`), then this block is not
                // called and the `router!` macro continues looking for another block.
                println!("u32 {:?}", id);

                // For the same of the example we return an empty response with a 400 status code.
                rouille::Response::empty_400()
            },

            (GET) (/{id: String}) => {
                // If the request's URL is for example `/foo`, we jump here.
                //
                // This route is similar to the previous one, but this time we have a `String`.
                // Parsing into a `String` never fails.
                println!("String {:?}", id);

                // Builds a `Response` object that contains "hello, " followed with the value
                // of `id`.
                rouille::Response::text(format!("hello, {}", id))
            },

            // The code block is called if none of the other blocks matches the request.
            // We return an empty response with a 404 status code.
            _ => rouille::Response::empty_404()
        )
    });
}
