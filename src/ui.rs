use std::{rc::Rc, cell::RefCell};

use gtk4 as gtk;
use gtk::{prelude::*, glib::clone};

use crate::game;

const BUTTON_SIZE: i32 = 16;

pub fn run() -> gtk::glib::ExitCode {
    let game = Rc::new(RefCell::new(game::Game::new(game::Level::Easy)));
    game.borrow_mut().start();

    let window_width = i32::from(game.borrow().count_cols()) * BUTTON_SIZE;
    let window_height = i32::from(game.borrow().count_rows()) * BUTTON_SIZE;

    let app = gtk::Application::builder()
        .application_id("ferris-minesweeper")
        .build();

    app.connect_activate(clone!(@strong game => move |app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .default_width(window_width)
            .default_height(window_height)
            .resizable(false)
            .title("Ferris Minesweeper")
            .build();

        let grid = gtk::Grid::builder().build();

        let rows = game.borrow().count_rows();
        let cols = game.borrow().count_cols();

        for x in 0..rows {
            for y in 0..cols {
                let button = gtk::ToggleButton::builder().build();
                let position = game::Position::new(i32::from(x), i32::from(y));
                unsafe {
                    button.set_data("position", Box::new(position));
                };

                button.connect_clicked(clone!(@strong game => move |button| {
                    button.set_sensitive(false);
                    unsafe {
                        let position: Option<std::ptr::NonNull<Box<game::Position>>> = button.data("position");
                        if let Some(position) = position {
                            let openned_positions = game.borrow_mut().open_square(position.as_ref());
                            println!("{:?}", position.as_ref());
                            println!("{:?}", openned_positions);
                        }
                    }
                }));

                grid.attach(&button, i32::from(y), i32::from(x), 1, 1);
            }
        }

        window.set_child(Some(&grid));
        window.show();
    }));

    app.run()
}