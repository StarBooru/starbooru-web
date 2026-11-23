//! Files the browser hands over: from a file picker, a drop (folders
//! included) or a paste. Only images and videos are kept, and a file is read
//! a piece at a time, never whole.

use bytes::Bytes;
use js_sys::{Array, Promise, Reflect, Uint8Array};
use starbooru_core::{SNIFF_BYTES, is_media};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    DataTransfer, File, FileList, FileSystemDirectoryEntry, FileSystemDirectoryReader,
    FileSystemEntry, FileSystemFileEntry,
};

use crate::logic::upload::{Verdict, drop_is_urls, drop_urls, verdict};

/// A file, and where it was in a folder that was picked or dropped
/// (`Cats/a.jpg`), or nothing for a file on its own.
#[derive(Clone, Debug)]
pub struct Picked {
    pub file: File,
    pub path: String,
}

/// The most files a dropped folder gives, so a drop of a whole disk ends.
const MAX_WALK: usize = 20_000;

/// The files of a picker or a paste, with the path each had in a picked folder.
pub fn listed(list: Option<FileList>) -> Vec<Picked> {
    let Some(list) = list else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.get(i))
        .map(|file| {
            // `webkitRelativePath`: web-sys has no getter for it.
            let path = Reflect::get(&file, &JsValue::from_str("webkitRelativePath"))
                .ok()
                .and_then(|p| p.as_string())
                .unwrap_or_default();
            Picked { file, path }
        })
        .collect()
}

/// Whether a drag carries files, or a link or a picture out of another page.
pub fn carries_files(data: &DataTransfer) -> bool {
    data.types()
        .iter()
        .filter_map(|t| t.as_string())
        .any(|t| t == "Files" || t == "text/uri-list")
}

/// What a drop carries. The folders in it are read afterwards (`walk`): the
/// drop event is the only moment its list can be asked for.
#[derive(Default)]
pub struct Dropped {
    pub files: Vec<Picked>,
    pub folders: Vec<FileSystemDirectoryEntry>,
    pub urls: Vec<String>,
}

impl Dropped {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.folders.is_empty() && self.urls.is_empty()
    }
}

/// Reads a drop, during the drop event.
pub fn read_drop(data: &DataTransfer) -> Dropped {
    let get = |kind: &str| data.get_data(kind).unwrap_or_default();
    let urls = drop_urls(&get("text/uri-list"), &get("text/html"), &get("text/plain"));
    let mut files = Vec::new();
    let mut folders = Vec::new();
    let items = data.items();
    for i in 0..items.length() {
        let Some(item) = items.get(i).filter(|item| item.kind() == "file") else {
            continue;
        };
        let folder = item
            .webkit_get_as_entry()
            .ok()
            .flatten()
            .filter(FileSystemEntry::is_directory)
            .and_then(|entry| entry.dyn_into::<FileSystemDirectoryEntry>().ok());
        if let Some(folder) = folder {
            folders.push(folder);
        } else if let Some(file) = item.get_as_file().ok().flatten() {
            files.push(Picked {
                file,
                path: String::new(),
            });
        }
    }
    if files.is_empty() && folders.is_empty() {
        files = listed(data.files());
    }
    if drop_is_urls(files.len().saturating_add(folders.len()), &urls) {
        Dropped {
            urls,
            ..Dropped::default()
        }
    } else {
        Dropped {
            files,
            folders,
            urls: Vec::new(),
        }
    }
}

/// Text of a JavaScript error, for saying why a file couldn't be read.
fn reason(e: &JsValue) -> String {
    Reflect::get(e, &JsValue::from_str("message"))
        .ok()
        .and_then(|m| m.as_string())
        .or_else(|| e.as_string())
        .unwrap_or_default()
}

/// One batch of a folder's entries; an empty one when there are no more.
async fn read_batch(reader: &FileSystemDirectoryReader) -> Vec<FileSystemEntry> {
    let promise = Promise::new(&mut |resolve, reject| {
        if let Err(e) = reader.read_entries_with_callback_and_callback(&resolve, &reject) {
            let _ = reject.call1(&JsValue::NULL, &e);
        }
    });
    let Ok(list) = JsFuture::from(promise).await else {
        return Vec::new();
    };
    Array::from(&list)
        .iter()
        .filter_map(|e| e.dyn_into::<FileSystemEntry>().ok())
        .collect()
}

async fn entry_file(entry: &FileSystemFileEntry) -> Option<File> {
    let promise = Promise::new(&mut |resolve, reject| {
        entry.file_with_callback_and_callback(&resolve, &reject);
    });
    JsFuture::from(promise).await.ok()?.dyn_into::<File>().ok()
}

/// Every file in the dropped folders and the folders in them, each with its
/// path (`/Cats/a.jpg`), in the order of their paths.
pub async fn walk(folders: Vec<FileSystemDirectoryEntry>) -> Vec<Picked> {
    let mut out = Vec::new();
    let mut left = folders;
    while let Some(folder) = left.pop() {
        let reader = folder.create_reader();
        loop {
            let batch = read_batch(&reader).await;
            if batch.is_empty() || out.len() >= MAX_WALK {
                break;
            }
            for entry in batch {
                let path = entry.full_path();
                if entry.is_directory() {
                    if let Ok(inner) = entry.dyn_into::<FileSystemDirectoryEntry>() {
                        left.push(inner);
                    }
                } else if let Ok(file_entry) = entry.dyn_into::<FileSystemFileEntry>()
                    && let Some(file) = entry_file(&file_entry).await
                {
                    out.push(Picked { file, path });
                }
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The first `len` bytes of a file, or `None` when it can't be read (a
/// folder dropped where a file was expected, a file gone since).
pub async fn head(file: &File, len: u32) -> Option<Vec<u8>> {
    let piece = file.slice_with_f64_and_f64(0.0, f64::from(len)).ok()?;
    let buffer = JsFuture::from(piece.array_buffer()).await.ok()?;
    Some(Uint8Array::new(&buffer).to_vec())
}

/// The images and videos among `picked`, and the names of the others. A
/// name or a type settles most; the rest count by what they hold.
pub async fn media_files(picked: Vec<Picked>) -> (Vec<Picked>, Vec<String>) {
    let mut media = Vec::with_capacity(picked.len());
    let mut skipped = Vec::new();
    for p in picked {
        let name = p.file.name();
        let size = p.file.size().max(0.0) as u64;
        let take = match verdict(&name, &p.file.type_(), size) {
            Verdict::Take => true,
            Verdict::Skip => false,
            Verdict::Look => head(&p.file, u32::from(SNIFF_BYTES))
                .await
                .is_some_and(|h| is_media(&h)),
        };
        if take {
            media.push(p);
        } else {
            skipped.push(name);
        }
    }
    (media, skipped)
}

/// A piece of a file, for one chunk of its upload.
pub async fn read_range(file: &File, offset: u64, len: u64) -> Result<Bytes, String> {
    let end = offset.saturating_add(len);
    let piece = file
        .slice_with_f64_and_f64(offset as f64, end as f64)
        .map_err(|e| reason(&e))?;
    let buffer = JsFuture::from(piece.array_buffer())
        .await
        .map_err(|e| reason(&e))?;
    Ok(Uint8Array::new(&buffer).to_vec().into())
}

/// An address the page can show a picked file at, until [`release`].
pub fn object_url(file: &File) -> Option<String> {
    web_sys::Url::create_object_url_with_blob(file).ok()
}

/// Lets the browser forget a file shown by [`object_url`].
pub fn release(url: &str) {
    let _ = web_sys::Url::revoke_object_url(url);
}
