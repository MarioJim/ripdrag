use std::str::FromStr;

use gtk::{
    gdk::DragAction,
    glib::{self, Continue, MainContext, Type, PRIORITY_DEFAULT},
    traits::WidgetExt,
    Button, DropTarget, ListBox,
};
use url::Url;

use crate::{
    cli::Cli,
    common_ui::{generate_buttons_from_paths, generate_compact},
};

pub fn build_target_ui(list_box: ListBox, args: Cli) {
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
    let mut paths = Vec::new();
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
