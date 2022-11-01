use clap::Parser;
use gtk::{
    gdk::Key,
    gio::ApplicationFlags,
    glib::{set_program_name, signal},
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::WidgetExt,
    Application, ApplicationWindow, EventControllerKey, ListBox, PolicyType, ScrolledWindow,
};

mod cli;
mod common_ui;
mod source;
mod target;

fn main() {
    set_program_name(Some("ripdrag"));
    let app = Application::builder()
        .application_id("ga.strin.ripdrag")
        .flags(ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(build_ui);
    app.run_with_args(&[""]); // we don't want gtk to parse the arguments. cleaner solutions are welcome
}

fn build_ui(app: &Application) {
    // Parse arguments and check if files exist
    let args = cli::Cli::parse();
    for path in &args.paths {
        assert!(
            path.exists(),
            "{0} : no such file or directory",
            path.display()
        );
    }
    // Create a scrollable list
    let list_box = ListBox::new();
    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never) //  Disable horizontal scrolling
        .min_content_width(args.content_width)
        .child(&list_box)
        .build();

    // Build the main window
    let window = ApplicationWindow::builder()
        .title("ripdrag")
        .resizable(args.resizable)
        .application(app)
        .child(&scrolled_window)
        .default_height(args.content_height)
        .build();

    if args.target {
        target::build_target_ui(list_box, args);
    } else {
        source::build_source_ui(list_box, args);
    }

    // Kill the app when Escape is pressed
    let event_controller = EventControllerKey::new();
    event_controller.connect_key_pressed(|_, key, _, _| {
        if key == Key::Escape {
            std::process::exit(0)
        }
        signal::Inhibit(false)
    });

    window.add_controller(&event_controller);
    window.show();
}
