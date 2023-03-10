pub mod application;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let application = application::Application::new();
    application.init()
}
