use notes_vercel_backend::handle_request;
use vercel_runtime::{Error, run, service_fn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handle_request)).await
}
