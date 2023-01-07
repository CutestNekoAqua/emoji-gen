use std::fs::File;
use std::fs::create_dir;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::rc::Rc;

use clap::Parser;
use clap::builder::OsStr;
use debug_print::debug_println;
use walkdir::WalkDir;
use walkdir::DirEntry;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use zip::result::ZipError;
use zip::write::FileOptions;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Folder with the custom emojis to generate the pack from.
   #[arg(short, long)]
   folder: String,

   /// Name for the pack
   #[arg(short, long, default_value_t = ("Custom".to_string()))]
   group: String,
}

#[derive(Serialize, Deserialize)]
struct Meta {
    metaVersion: i8,
	host: String,
	/**
	 * Date and time representation returned by ECMAScript `Date.prototype.toString`.
	 */
	exportedAt: String,
	emojis: Vec<Emoji>,
}

#[derive(Serialize, Deserialize)]
struct Emoji {
	downloaded: bool,
	fileName: String,
	emoji: EmojiData,
}

#[derive(Serialize, Deserialize)]
struct EmojiData {
    name: String,
    category: String,
    aliases: Vec<String>
}


fn main() {
    let args = Args::parse();
    
    let mut emojis = Vec::<Emoji>::new();
    let group = Rc::new(args.group);

    for result in WalkDir::new(args.folder).into_iter() {
        if let Err(_) = result {
            continue;
        }
        let opt_file = result.ok();
        if opt_file.is_none() {
            continue;
        }
        let file = opt_file.unwrap();
        if !file.metadata().unwrap().is_file() {
            continue;
        }

        let subcat = file.path().to_string_lossy()
            .replace(file.path().file_name().unwrap().to_string_lossy().to_string().as_str(), "")
            .replace("/", "")
            .replace(".", "");
        debug_println!("{}", file.path().file_name().unwrap().to_str().unwrap());

        let emoji = owoifier(file, group.clone(), subcat);
        if emoji.is_some() {
            emojis.push(emoji.unwrap());
        }
    }

    let meta = Meta {
        metaVersion: 1,
        host: "https://github.com/waterdev/owoifier".to_string(),
        exportedAt: "".to_string(),
        emojis: emojis,
    };

    let json = serde_json::to_string(&meta).unwrap();
    let mut file = File::create("meta.json").unwrap();
    write!(file, "{}", json);
    zip(".", "../generated_emotes.zip");
    println!("✅ Done! Importable ZIP file under '../generated_emotes.zip'");
}

fn owoifier(file: DirEntry, original_category: Rc<String>, subcategory: String) -> Option<Emoji> {
    debug_println!("{}", file.path().display());

    let path = file.path();
    let image = imghdr::from_file(file.path()).unwrap();
    if image.is_none() {
        return None
    }
    let mut name = file.file_name().to_ascii_lowercase().into_string().unwrap();
    name = str::replace(&name, file.path().extension().unwrap().to_string_lossy().to_string().as_str(), "");
    name = str::replace(&name, " ", "_");
    name = str::replace(&name, "-", "_");
    name = str::replace(&name, ".", "");

    let data = EmojiData{
        name: name,
        category: original_category.to_string() + " - " + subcategory.as_str(),
        aliases: Vec::<String>::new()};
    
    Some(Emoji {
        downloaded: true,
        fileName: path.to_string_lossy().into(),
        emoji: data
    })
}

fn zip(
    src_dir: &str,
    dst_file: &str,
) -> zip::result::ZipResult<()> {
    if !std::path::Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = std::path::Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, zip::CompressionMethod::Deflated)?;

    Ok(())
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(std::path::Path::new(prefix)).unwrap();
         // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            debug_println!("adding file {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            debug_println!("adding dir {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}