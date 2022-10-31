use std::{
    io::{self, BufRead},
    path::{Path, PathBuf},
    str::FromStr,
    thread,
};

use clap::Parser;
use gtk::{
    gdk::{ContentProvider, DragAction},
    gio::ApplicationFlags,
    glib::{self, Bytes, Continue, MainContext, Type, PRIORITY_DEFAULT},
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::{ButtonExt, WidgetExt},
    Application, ApplicationWindow, Button, CenterBox, DragSource, DropTarget, EventControllerKey,
    Image, ListBox, Orientation, PolicyType, ScrolledWindow,
};
use url::Url;

mod cli;

fn main() {
    glib::set_program_name(Some("ripdrag"));
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
        build_target_ui(list_box, args);
    } else {
        build_source_ui(list_box, args);
    }

    // Kill the app when Escape is pressed
    let event_controller = EventControllerKey::new();
    event_controller.connect_key_pressed(|_, key, _, _| {
        if key.name().unwrap() == "Escape" {
            std::process::exit(0)
        }
        glib::signal::Inhibit(false)
    });

    window.add_controller(&event_controller);
    window.show();
}

fn build_source_ui(list_box: ListBox, args: cli::Cli) {
    // Populate the list with the buttons, if there are any
    if !args.paths.is_empty() {
        if args.all_compact {
            list_box.append(&generate_compact(args.paths.clone(), args.and_exit));
        } else {
            for button in generate_buttons_from_paths(
                args.paths.clone(),
                args.and_exit,
                args.icons_only,
                args.disable_thumbnails,
                args.icon_size,
                args.all,
            ) {
                list_box.append(&button);
            }
        }
    }

    // Read from stdin and populate the list
    if args.from_stdin {
        let mut paths: Vec<PathBuf> = args.paths.clone();
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        thread::spawn(move || {
            let stdin = io::stdin();
            let lines = stdin.lock().lines();

            for line in lines {
                let path = PathBuf::from(line.unwrap());
                if path.exists() {
                    println!("Adding: {}", path.display());
                    sender.send(path).expect("Error");
                } else if args.verbose {
                    println!("{} : no such file or directory", path.display())
                }
            }
        });
        receiver.attach(
            None,
            glib::clone!(@weak list_box => @default-return Continue(false),
                move |path| {
                    if args.all_compact {
                        paths.push(path);
                        if let Some(child) = list_box.first_child() {
                            list_box.remove(&child);
                        }
                        list_box.append(&generate_compact(paths.clone(), args.and_exit));
                    } else {
                        let button = generate_buttons_from_paths(
                            vec![path],
                            args.and_exit,
                            args.icons_only,
                            args.disable_thumbnails,
                            args.icon_size,
                            args.all,
                        );
                        list_box.append(&button[0]);
                    }
                    Continue(true)
                }
            ),
        );
    }
}

fn build_target_ui(list_box: ListBox, args: cli::Cli) {
    // Generate the Drop Target and button
    let button = Button::builder().label("Drop your files here").build();

    let drop_target = DropTarget::new(Type::INVALID, DragAction::COPY);
    // TODO: This is broken on anything other than linux
    // Figure out a way to accept G_TYPE_FILE other than STRING
    drop_target.set_types(&[Type::STRING]);

    let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

    drop_target.connect_drop(move |_, value, _, _| {
        match value.get::<String>() {
            Ok(data) => {
                // safely extract the path string from the value
                if args.print_path {
                    data.lines().for_each(|line| match Url::from_str(line) {
                        Ok(url) => match url.to_file_path() {
                            Ok(path) => {
                                println!("{}", path.canonicalize().unwrap().to_string_lossy());
                            }
                            Err(_) => {
                                if args.verbose {
                                    println!("Cannot convert path to string");
                                }
                            }
                        },
                        Err(_) => {
                            if args.verbose {
                                println!("Cannot convert drop data to url");
                            }
                        }
                    });
                } else {
                    data.lines().for_each(|line| println!("{}", line));
                }
                if args.keep {
                    sender.send(data).expect("Error");
                }
            }
            Err(_) => {
                if args.verbose {
                    println!("Cannot decode drop data");
                }
            }
        };
        true
    });

    // get the uri_list from the drop and populate the list of files (--keep)
    let mut paths: Vec<PathBuf> = Vec::new();
    receiver.attach(
        None,
        glib::clone!(@weak list_box => @default-return Continue(false),
            move |uri_list| {
                let mut new_paths = Vec::new();
                uri_list.lines().for_each(|uri| {
                    if let Ok(url) = Url::from_str(uri) {
                        if let Ok(path) = url.to_file_path() {
                            new_paths.push(path);
                        }
                    }
                });
                if args.all_compact{
                    // Hacky solution, check if we already created buttons
                    if !paths.is_empty() {
                        if let Some(child) = list_box.last_child() {
                            list_box.remove(&child);
                        }
                    }
                    paths.append(&mut new_paths);
                    list_box.append(&generate_compact(paths.clone(), args.and_exit));
                } else {
                    // This solution is fast, but it's gonna cause problems when --all is used in combinatio with --target
                    for button in &generate_buttons_from_paths(
                        new_paths,
                        args.and_exit,
                        args.icons_only,
                        args.disable_thumbnails,
                        args.icon_size,
                        args.all,
                    ) {
                        list_box.append(button);
                    }
                }
                Continue(true)
            }
        ),
    );
    button.add_controller(&drop_target);
    list_box.append(&button);
}

fn generate_buttons_from_paths(
    paths: Vec<PathBuf>,
    and_exit: bool,
    icons_only: bool,
    disable_thumbnails: bool,
    icon_size: i32,
    all: bool,
) -> Vec<Button> {
    let mut button_vec = Vec::new();
    let uri_list = generate_uri_list(&paths);

    //TODO: make this loop multithreaded
    for path in paths.into_iter() {
        // The CenterBox(button_box) contains the image and the optional label
        // The Button contains the CenterBox and can be dragged
        let button_box = CenterBox::builder()
            .orientation(Orientation::Horizontal)
            .build();

        if let Some(image) = get_image_from_path(&path, icon_size, disable_thumbnails) {
            if icons_only {
                button_box.set_center_widget(Some(&image));
            } else {
                button_box.set_start_widget(Some(&image));
            }
        }

        if !icons_only {
            button_box.set_center_widget(Some(
                &gtk::Label::builder()
                    .label(path.display().to_string().as_str())
                    .build(),
            ));
        }

        let button = Button::builder().child(&button_box).build();
        let drag_source = DragSource::builder().build();

        if all {
            let list = uri_list.clone();
            drag_source.connect_prepare(move |_, _, _| {
                Some(ContentProvider::for_bytes("text/uri-list", &list))
            });
        } else {
            let uri = generate_uri_from_path(&path);
            drag_source.connect_prepare(move |_, _, _| {
                Some(ContentProvider::for_bytes("text/uri-list", &uri))
            });
        }

        if and_exit {
            drag_source.connect_drag_end(|_, _, _| std::process::exit(0));
        }

        // Open the path with the default app
        button.connect_clicked(move |_| {
            opener::open(&path).unwrap();
        });

        button.add_controller(&drag_source);
        button_vec.push(button);
    }
    button_vec
}

fn generate_compact(paths: Vec<PathBuf>, and_exit: bool) -> Button {
    // Here we want to generate a single draggable button, containg all the files
    let button = Button::builder()
        .label(&format!("{} elements", paths.len()))
        .build();
    let drag_source = DragSource::builder().build();

    drag_source.connect_prepare(move |_, _, _| {
        Some(ContentProvider::for_bytes(
            "text/uri-list",
            &generate_uri_list(&paths),
        ))
    });

    if and_exit {
        drag_source.connect_drag_end(|_, _, _| std::process::exit(0));
    }
    button.add_controller(&drag_source);
    button
}

fn get_image_from_path(
    path: &std::path::PathBuf,
    icon_size: i32,
    disable_thumbnails: bool,
) -> Option<Image> {
    let mime_type = if path.metadata().unwrap().is_dir() {
        "inode/directory"
    } else {
        match infer::get_from_path(path) {
            Ok(option) => match option {
                Some(infer_type) => infer_type.mime_type(),
                None => "text/plain",
            },
            Err(_) => "text/plain",
        }
    };
    if mime_type.contains("image") & !disable_thumbnails {
        Some(
            Image::builder()
                .file(path.as_os_str().to_str().unwrap())
                .pixel_size(icon_size)
                .build(),
        )
    } else {
        gtk::gio::content_type_get_generic_icon_name(mime_type).map(|icon_name| {
            Image::builder()
                .icon_name(&icon_name)
                .pixel_size(icon_size)
                .build()
        })
    }
}

fn generate_uri_from_path(path: &Path) -> Bytes {
    Bytes::from_owned(uri_from_path(path))
}

fn generate_uri_list(paths: &[PathBuf]) -> Bytes {
    let uris = paths
        .iter()
        .map(|path| uri_from_path(path))
        .reduce(|accum, item| [accum, item].join("\n"))
        .unwrap();

    Bytes::from_owned(uris)
}

fn uri_from_path(path: &Path) -> String {
    Url::from_file_path(path.canonicalize().unwrap())
        .unwrap()
        .to_string()
}
