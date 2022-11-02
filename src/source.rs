use std::{
    io::{self, BufRead},
    path::PathBuf,
    thread,
};

use gtk::{
    glib::{self, Continue, MainContext, PRIORITY_DEFAULT},
    traits::WidgetExt,
    ListBox,
};

use crate::{
    cli::Cli,
    common_ui::{generate_buttons_from_paths, generate_compact},
};

pub fn build_source_ui(list_box: ListBox, args: Cli) {
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
        let mut paths = args.paths.clone();
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
                        ).next().unwrap();
                        list_box.append(&button);
                    }
                    Continue(true)
                }
            ),
        );
    }
}
