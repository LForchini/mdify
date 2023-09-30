use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
};

use clap::Parser;
use comrak::{
    markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
    ComrakRenderOptions,
};
use pathdiff::diff_paths;
use regex::{Captures, Regex};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    /// Optional stylesheet to bake into the HTML
    stylesheet: Option<PathBuf>,

    #[arg(short, long, default_value = "build")]
    /// Directory to place HTML files into
    out_dir: PathBuf,

    #[arg()]
    /// Directory of MD files to transform
    src_dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    if args.out_dir.canonicalize().is_err() {
        create_dir_all(&args.out_dir).expect("should be able to create missing build folder");
    }

    let options = ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: true,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            header_ids: Some("header-".to_string()),
            footnotes: true,
            ..Default::default()
        },
        parse: ComrakParseOptions {
            smart: true,
            relaxed_tasklist_matching: true,
            ..Default::default()
        },
        render: ComrakRenderOptions {
            ..Default::default()
        },
    };

    let re = Regex::new(r"\[(?<text>.*)\]\((?<link>.*)\.md\)").unwrap();

    let styling = args
        .stylesheet
        .and_then(|p| read_to_string(p).ok())
        .map(|s| format!("<style>{}</style>", s));

    let html_autoinclude = include_str!("autoinclude.html");

    let src_dir = args.src_dir.canonicalize().unwrap();
    let mut glob = src_dir.clone();
    glob.push("**");
    glob.push("*.md");

    let _ = glob::glob(&glob.to_string_lossy())
        .expect("input string should be a valid globbable path")
        .filter_map(|p| {
            let p = p.ok()?;

            let relative_path = diff_paths(&p, &src_dir)?;
            let mut out_path = args.out_dir.canonicalize().unwrap();
            out_path.extend(&relative_path);
            out_path.set_extension("html");

            let content = read_to_string(&p).ok()?;
            let content = re.replace_all(&content, |caps: &Captures<'_>| {
                format!("[{}]({}.html)", &caps["text"], &caps["link"])
            });
            let html = markdown_to_html(&content, &options);

            create_dir_all(out_path.parent()?).ok()?;
            let mut out_file = File::create(out_path).ok()?;
            out_file.write_all(html.as_bytes()).ok()?;

            if let Some(styling) = &styling {
                out_file.write_all(styling.as_bytes()).ok()?;
            }

            out_file.write_all(html_autoinclude.as_bytes()).ok()?;

            Some(())
        })
        .collect::<Vec<_>>();
}
