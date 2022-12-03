use gtk::prelude::*;

mod gaussian_plot;
mod window;

fn main() {
    let application = gtk::Application::new(
        Some("io.github.plotters-rs.plotters-gtk-demo"),
        Default::default(),
    );

    application.connect_activate(|app| {
        let win = window::Window::new(app);
        win.show();
    });

    application.run();
}
