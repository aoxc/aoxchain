use aoxchub::app::App;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = App::bootstrap().await?;
    app.run().await
}
