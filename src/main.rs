use clap::Parser;
use jwalk::{DirEntry, WalkDir};
use read_fonts::TableProvider;
use skrifa::{FontRef, MetadataProvider};
#[derive(Parser, Debug)]
struct Args {
    /// Variation axes to find
    #[arg(short, long)]
    axis: Vec<String>,

    /// Codepoints to find (comma-separated list of hyphen-delimited ranges)
    #[arg(short, long, value_parser = parse_unicode_ranges)]
    codepoint: Vec<Vec<u32>>,

    /// OpenType features to find
    #[arg(short, long)]
    feature: Vec<String>,

    /// Scripts to find
    #[arg(short, long)]
    script: Vec<String>,

    /// Name table entries to find
    #[arg(short, long)]
    name: Vec<String>,

    /// Directory to search for fonts
    #[arg(default_value = ".")]
    directory: String,
}

fn parse_unicode_ranges(arg: &str) -> Result<Vec<u32>, String> {
    let mut codepoints = Vec::new();
    for range in arg.to_ascii_uppercase().split(',') {
        let parts: Vec<&str> = range
            .split('-')
            .map(|part| {
                if part.starts_with("U+") || part.starts_with("0x") {
                    &part[2..]
                } else {
                    part
                }
            })
            .collect();
        if parts.len() == 1 {
            codepoints.push(u32::from_str_radix(parts[0], 16).map_err(|e| e.to_string())?);
        } else if parts.len() == 2 {
            let start = u32::from_str_radix(parts[0], 16).map_err(|e| e.to_string())?;
            let end = u32::from_str_radix(parts[1], 16).map_err(|e| e.to_string())?;
            for codepoint in start..=end {
                codepoints.push(codepoint);
            }
        } else {
            return Err(format!("Bad range: {}", range));
        }
    }
    Ok(codepoints)
}

fn feature_filter(font: &FontRef, feature: &str) -> bool {
    let gsub_featurelist = font.gsub().ok().and_then(|gsub| gsub.feature_list().ok());
    let gpos_feature_list = font.gpos().ok().and_then(|gpos| gpos.feature_list().ok());
    let gsub_feature_and_data = gsub_featurelist.map(|list| {
        list.feature_records()
            .iter()
            .map(move |feature| (feature, feature.feature(list.offset_data())))
    });
    let gpos_feature_and_data = gpos_feature_list.map(|list| {
        list.feature_records()
            .iter()
            .map(move |feature| (feature, feature.feature(list.offset_data())))
    });
    gsub_feature_and_data
        .into_iter()
        .flatten()
        .chain(gpos_feature_and_data.into_iter().flatten())
        .any(|(f, _)| f.feature_tag() == feature)
}

fn axis_filter(font: &FontRef, axis: &str) -> bool {
    font.axes().iter().any(|a| a.tag() == axis)
}

fn script_filter(font: &FontRef, script: &str) -> bool {
    let gsub_script_list = font.gsub().ok().and_then(|gsub| gsub.script_list().ok());
    let gpos_script_list = font.gpos().ok().and_then(|gpos| gpos.script_list().ok());
    let gsub_script_and_data = gsub_script_list.map(|list| {
        list.script_records()
            .iter()
            .map(move |script| (script, script.script(list.offset_data())))
    });
    let gpos_script_and_data = gpos_script_list.map(|list| {
        list.script_records()
            .iter()
            .map(move |script| (script, script.script(list.offset_data())))
    });
    gsub_script_and_data
        .into_iter()
        .flatten()
        .chain(gpos_script_and_data.into_iter().flatten())
        .any(|(s, _)| s.script_tag() == script)
}

fn codepoint_filter(font: &FontRef, codepoint: u32) -> bool {
    font.charmap().map(codepoint).is_some()
}

fn name_filter(font: &FontRef, needle: &str) -> bool {
    let Ok(name) = font.name() else {
        return false;
    };
    let records = name.name_record().iter();
    records
        .flat_map(|record| record.string(name.string_data()))
        .any(|s| s.chars().collect::<String>().contains(needle))
}

type StringFilter = dyn Fn(&FontRef, &str) -> bool;

fn filter_font(entry: &DirEntry<((), ())>, args: &Args) -> Result<bool, ()> {
    let name = entry.file_name().to_str().ok_or(())?;
    if !name.ends_with(".otf") && !name.ends_with(".ttf") {
        return Ok(false);
    }
    let data = std::fs::read(entry.path()).map_err(|_| ())?;
    let font = FontRef::new(&data).map_err(|_| ())?;
    let filters: Vec<(&StringFilter, &Vec<String>)> = vec![
        (&feature_filter, &args.feature),
        (&axis_filter, &args.axis),
        (&script_filter, &args.script),
        (&name_filter, &args.name),
    ];

    for (filter, values) in filters {
        for value in values.iter() {
            if !filter(&font, value) {
                return Ok(false);
            }
        }
    }

    for codepoint in args.codepoint.iter().flatten() {
        if !codepoint_filter(&font, *codepoint) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn main() {
    let args = Args::parse();
    let directory = args.directory.clone();
    let walker = WalkDir::new(directory)
        .process_read_dir(move |_depth, _path, _read_dir_state, children| {
            children.retain(|dir_entry_result| {
                dir_entry_result
                    .as_ref()
                    .map(|dir_entry| {
                        dir_entry.file_type().is_dir()
                            || filter_font(dir_entry, &args).unwrap_or(false)
                    })
                    .unwrap_or(false)
            });
        })
        .sort(true);

    for entry in walker.into_iter().flatten() {
        if entry.file_type().is_dir() {
            continue;
        }
        println!("{}", entry.path().display());
    }
}
