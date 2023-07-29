use std::{ptr::NonNull, rc::Rc, cell::RefCell};

use gtk4 as gtk;
use gtk::{prelude::*, glib::clone};

use crate::game::{self, OpennedPosition};

const BUTTON_SIZE: i32 = 16;

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("../res/styles.css"));

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn open_positions(positions: Vec<OpennedPosition>, grid: &gtk::Grid) {
    for openned_pos in positions {
        let position = openned_pos.position;
        let button = grid.child_at(
            position.y,
            position.x,
        );
        if button.is_none() {
            continue;
        }
        let button = button.unwrap().downcast::<gtk::Button>().unwrap();
        let even = (position.x + position.y) % 2 == 0;
        let mut css_classes = Vec::new();
        if even {
            css_classes.push("even-open");
        } else {
            css_classes.push("odd-open");
        }
        if openned_pos.adjacent_mines > 0 {
            let css_class = match openned_pos.adjacent_mines {
                1 => "one",
                2 => "two",
                3 => "three",
                4 => "four",
                5 => "five",
                6 => "six",
                7 => "seven",
                8 => "eight",
                _ => "",
            };
            css_classes.push(css_class);
            button.set_label(openned_pos.adjacent_mines.to_string().as_str());
        }
        if openned_pos.mined {
            button.set_icon_name("face-worried");
        }
        button.set_css_classes(&css_classes);
        button.set_sensitive(false);
    }
}

fn button_on_click(game: &Rc<RefCell<game::Game>>, button: &gtk::Button) {
    if game.borrow().over() {
        return;
    }
    let position: Option<NonNull<Box<game::Position>>>;
    unsafe {
        position = button.data("position");
    }
    if position.is_none() {
        return;
    }
    let position = position.unwrap();
    let positions: Vec<OpennedPosition>;
    unsafe {
        let position = game::Position::new(position.as_ref().x ,position.as_ref().y);
        positions = game.borrow_mut().open_position(position);
    }
    if button.parent().is_none() {
        return;
    }
    if let Some(parent) = button.parent() {
        let grid = parent.downcast::<gtk::Grid>().unwrap();
        open_positions(positions, &grid);
    }
}

fn create_right_click_handler(game: &Rc<RefCell<game::Game>>, button: &gtk::Button, position: game::Position) -> gtk::GestureClick {
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_SECONDARY as u32);
    gesture.connect_pressed(clone!(@strong game, @strong button => move |_, _, _, _| {
        let tag = game.borrow_mut().tag_position(game::Position::new(position.x, position.y));
        match tag {
            game::Tag::None => {
                button.set_icon_name("");
            },
            game::Tag::Flag => {
                button.set_icon_name("face-uncertain");
            },
            game::Tag::Question => {
                button.set_icon_name("");
                button.set_label("?");
            },
        };
    }));
    gesture
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
        button_on_click(&game, button);
    }));
    button.add_controller(create_right_click_handler(game, &button, position));
    button
}

fn build_board(game: &Rc<RefCell<game::Game>>) -> gtk::Grid {
    let grid = gtk::Grid::builder().build();

    for x in 0..game.borrow().rows() {
        for y in 0..game.borrow().cols() {
            let position = game::Position::new(i32::from(x), i32::from(y));
            let button = build_square(game, position);
            grid.attach(&button, i32::from(y), i32::from(x), 1, 1);
        }
    }

    grid
}

fn build_ui(app: &gtk::Application) {
    let game = Rc::new(RefCell::new(game::Game::new(game::Level::Hard)));
    game.borrow_mut().start();

    let window_width = i32::from(game.borrow().cols()) * BUTTON_SIZE;
    let window_height = i32::from(game.borrow().rows()) * BUTTON_SIZE;

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .default_width(window_width)
        .default_height(window_height)
        .resizable(false)
        .title("Ferris Minesweeper")
        .build();

    let grid = build_board(&game);
    window.set_child(Some(&grid));
    window.show();
}

pub fn run() -> gtk::glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("com.josimarz.ferris-minesweeper")
        .build();

    app.connect_startup(|_| { load_css(); });
    app.connect_activate(build_ui);

    app.run()
}