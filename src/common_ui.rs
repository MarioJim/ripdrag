use std::path::{Path, PathBuf};

use gtk::{
    gdk::ContentProvider,
    glib::Bytes,
    traits::{ButtonExt, WidgetExt},
    Button, CenterBox, DragSource, Image, Orientation,
};
use url::Url;

pub fn generate_compact(paths: Vec<PathBuf>, and_exit: bool) -> Button {
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

pub fn generate_buttons_from_paths(
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
