use aoxchub::app::App;

#[tokio::main]
async fn main() {
    let app = App::bootstrap().await.expect("AOXCHub bootstrap failed");
    app.run().await.expect("AOXCHub server failure");
}
