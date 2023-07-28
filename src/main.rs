use gtk4 as gtk;

mod game;
mod ui;

fn main() -> gtk::glib::ExitCode {
    ui::run()
}
