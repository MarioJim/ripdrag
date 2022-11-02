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
    let uri_bytes = uri_list_to_bytes(&paths);
    button.add_controller(&drag_source_for_uri_bytes(uri_bytes, and_exit));
    button
}

pub fn generate_buttons_from_paths(
    paths: Vec<PathBuf>,
    and_exit: bool,
    icons_only: bool,
    disable_thumbnails: bool,
    icon_size: i32,
    all: bool,
) -> impl Iterator<Item = Button> {
    let uri_list = uri_list_to_bytes(&paths);

    // TODO: make this loop multithreaded
    paths.into_iter().map(move |path| {
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

        let uri_bytes = if all {
            uri_list.clone()
        } else {
            uri_list_to_bytes(&[path.clone()])
        };
        let button = Button::builder().child(&button_box).build();
        button.add_controller(&drag_source_for_uri_bytes(uri_bytes, and_exit));

        // Open the path with the default app
        button.connect_clicked(move |_| {
            if let Err(error) = opener::open(&path) {
                eprintln!("Error opening {}:\n{}", path.display(), error);
            }
        });

        button
    })
}

fn get_image_from_path(path: &Path, icon_size: i32, disable_thumbnails: bool) -> Option<Image> {
    let mime_type = if path.metadata().unwrap().is_dir() {
        "inode/directory"
    } else {
        infer::get_from_path(path)
            .ok()
            .flatten()
            .map_or("text/plain", |infer_type| infer_type.mime_type())
    };

    if mime_type.contains("image") & !disable_thumbnails {
        let image = Image::builder()
            .file(path.to_string_lossy().as_ref())
            .pixel_size(icon_size)
            .build();
        Some(image)
    } else {
        gtk::gio::content_type_get_generic_icon_name(mime_type).map(|icon_name| {
            Image::builder()
                .icon_name(&icon_name)
                .pixel_size(icon_size)
                .build()
        })
    }
}

fn drag_source_for_uri_bytes(uri_bytes: Bytes, and_exit: bool) -> DragSource {
    let drag_source = DragSource::builder().build();

    drag_source.connect_prepare(move |_, _, _| {
        Some(ContentProvider::for_bytes("text/uri-list", &uri_bytes))
    });

    if and_exit {
        drag_source.connect_drag_end(|_, _, _| std::process::exit(0));
    }

    drag_source
}

fn uri_list_to_bytes(paths: &[PathBuf]) -> Bytes {
    let uris_string = paths
        .iter()
        .map(|path| {
            let canonicalized_path = path
                .canonicalize()
                .expect("Error getting the an absolute path");
            Url::from_file_path(canonicalized_path).unwrap().to_string()
        })
        .collect::<Vec<String>>()
        .join("\n");
    Bytes::from_owned(uris_string)
}
