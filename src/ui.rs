use std::ptr::NonNull;
use std::{rc::Rc, cell::RefCell};

use gtk4 as gtk;
use gtk::{prelude::*, glib::clone};
use gtk::glib::object::Cast;

use crate::game;

const BUTTON_SIZE: i32 = 16;

fn get_css_class(adjacent_mines: usize) -> &'static str {
    match adjacent_mines {
        1 => "one",
        2 => "two",
        3 => "three",
        4 => "four",
        5 => "five",
        6 => "six",
        7 => "seven",
        8 => "eight",
        _ => "",
    }
}

fn open_square(game: &Rc<RefCell<game::Game>>, button: &gtk::Button) {
    if game.borrow().is_over() {
        return;
    }
    unsafe {
        let position: Option<NonNull<Box<game::Position>>> = button.data("position");
        if let Some(position) = position {
            let openned_positions = game.borrow_mut().open_square(position.as_ref());
            let found = openned_positions.iter().find(|open| open.equals(position.as_ref()));
            if found.is_some() {
                button.set_sensitive(false);
                let openned_positions = openned_positions.iter().filter(|open| !open.equals(position.as_ref())).collect::<Vec<_>>();
                let even = (position.as_ref().x + position.as_ref().y) % 2 == 0;
                let mut css_classes = Vec::new();
                if even {
                    css_classes.push("even-open");
                } else {
                    css_classes.push("odd-open");
                }
                let adjacent_mines = game.borrow_mut().count_adjacent_mines(position.as_ref());
                if game.borrow().is_mined(position.as_ref()) {
                    css_classes.push("mine");
                    button.set_icon_name("face-worried");
                } else if adjacent_mines > 0 {
                    css_classes.push(get_css_class(adjacent_mines));
                    button.set_label(adjacent_mines.to_string().as_str());
                }
                button.set_css_classes(&css_classes);
                if let Some(parent) = button.parent() {
                    let grid = parent.downcast::<gtk::Grid>().unwrap();
                    openned_positions.iter()
                        .for_each(|position| {
                            let child = grid.child_at(position.y, position.x);
                            if let Some(child) = child {
                                let child = child.downcast::<gtk::Button>().unwrap();
                                open_square(game, &child);
                            }
                        });
                }
            }
        }
    }
}

fn handle_right_click(game: &Rc<RefCell<game::Game>>, button: &gtk::Button, position: game::Position) {
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
    gesture.connect_pressed(clone!(@strong game, @strong button => move |_, _, _, _| {
        let tag = game.borrow_mut().tag_square(&position);
        let icon_name = match tag {
            game::Tag::None => "",
            game::Tag::Flag => "face-smile",
            game::Tag::Question => "face-uncertain"
        };
        button.set_icon_name(icon_name);
    }));
    button.add_controller(gesture);
}

fn build_square(game: &Rc<RefCell<game::Game>>, position: game::Position) -> gtk::Button {
    let even = (position.x + position.y) % 2 == 0;
    let css_classes: Vec<&str>;
    if even {
        css_classes = vec!["even-closed"];
    } else {
        css_classes = vec!["odd-closed"];
    }
    let button = gtk::Button::builder()
        .css_classes(css_classes)
        .build();
    unsafe {
        button.set_data("position", Box::new(position));
    }
    button.connect_clicked(clone!(@strong game => move |button| {
        open_square(&game, &button);
    }));
    button
}

fn build_squares(game: &Rc<RefCell<game::Game>>, grid: &gtk::Grid) {
    let rows = game.borrow().count_rows();
    let cols = game.borrow().count_cols();

    for x in 0..rows {
        for y in 0..cols {
            let position = game::Position::new(i32::from(x), i32::from(y));
            let button = build_square(game, position);
            handle_right_click(game, &button, position);
            grid.attach(&button, i32::from(y), i32::from(x), 1, 1);
        }
    }
}

fn build_board(game: &Rc<RefCell<game::Game>>, window: &gtk::ApplicationWindow) {
    let grid = gtk::Grid::builder().build();
    build_squares(game, &grid);
    window.set_child(Some(&grid));
}

fn build_window(app: &gtk::Application) {
    let game = Rc::new(RefCell::new(game::Game::new(game::Level::Hard)));
    game.borrow_mut().start();

    let window_width = i32::from(game.borrow().count_cols()) * BUTTON_SIZE;
    let window_height = i32::from(game.borrow().count_rows()) * BUTTON_SIZE;

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .default_width(window_width)
        .default_height(window_height)
        .resizable(false)
        .title("Ferris Minesweeper")
        .build();

    build_board(&game, &window);
    window.show();
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("../res/styles.css"));

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display"),
        &provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn run() -> gtk::glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("ferris-minesweeper")
        .build();
    app.connect_startup(|_| { load_css() });
    app.connect_activate(build_window);
    app.run()
}