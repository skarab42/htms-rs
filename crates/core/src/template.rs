use std::{
    collections::BTreeSet,
    fs,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    result,
};

use lol_html::{
    HtmlRewriter, Settings, element,
    errors::RewritingError,
    html_content::{ContentType, Element, EndTag},
};
use syn::{Ident, parse_str};

static CHUCK_BUFFER_SIZE: usize = 16 * 1024;
static STATIC_STYLE_CSS: &str = include_str!("static/style.css");
static STATIC_ON_RESPONSE_JS: &str = include_str!("static/on_response.js");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to open input path: {0}: {1}")]
    OpenInputPath(PathBuf, #[source] io::Error),
    #[error("failed to create output directory: {0}: {1}")]
    CreateOutputDirectory(PathBuf, #[source] io::Error),
    #[error("failed to create output file: {0}: {1}")]
    CreateOutputFile(PathBuf, #[source] io::Error),
    #[error("failed to read input chunk: {0}: {1}")]
    ReadInputChunk(PathBuf, #[source] io::Error),
    #[error("failed to rewrite input chunk: {0}: {1}")]
    RewriterWrite(PathBuf, #[source] RewritingError),
    #[error("failed to close rewriter: {0}: {1}")]
    RewriterEnd(PathBuf, #[source] RewritingError),
    #[error("failed to flush output file: {0}: {1}")]
    FlushOutputFile(PathBuf, #[source] io::Error),
    #[error("invalid attribute '{tag}[data-htms]' at byte offset {offset}: {source}")]
    InvalidHtmsAttribute {
        tag: String,
        offset: usize,
        source: syn::Error,
    },
}

pub type Result<T, E = Error> = result::Result<T, E>;

pub type TaskNames = BTreeSet<String>;

// TODO: add documentation
#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn parse_and_build<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<TaskNames> {
    let input_path = input_path.as_ref();
    let mut input_file =
        File::open(input_path).map_err(|error| Error::OpenInputPath(input_path.into(), error))?;

    let output_path = output_path.as_ref();
    if let Some(output_directory) = output_path.parent() {
        fs::create_dir_all(output_directory)
            .map_err(|error| Error::CreateOutputDirectory(output_directory.into(), error))?;
    }

    let mut output_file = File::create(output_path)
        .map_err(|error| Error::CreateOutputFile(output_path.into(), error))?;

    let output_sink = |c: &[u8]| {
        // TODO: handle errors?
        #[allow(clippy::expect_used)]
        output_file.write_all(c).expect("write chunk to file");
    };

    let mut task_names = BTreeSet::new();

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("html>head", |el| {
                    el.append("<style>", ContentType::Html);
                    el.append(STATIC_STYLE_CSS, ContentType::Html);
                    el.append("</style>", ContentType::Html);

                    Ok(())
                }),
                element!(r#"[data-htms^="fn:"]"#, |el| {
                    let attribute_value = el.get_attribute("data-htms").unwrap_or_default();
                    let (_, method_name) =
                        attribute_value.trim().split_once(':').unwrap_or_default();

                    if let Err(source) = parse_str::<Ident>(method_name) {
                        return Err(Error::InvalidHtmsAttribute {
                            tag: el.tag_name(),
                            offset: el.source_location().bytes().start,
                            source,
                        }
                        .into());
                    }

                    el.set_attribute("data-htms", method_name)?;
                    task_names.insert(method_name.to_string());

                    Ok(())
                }),
                element!("body", |el: &mut Element| {
                    el.append("<script>", ContentType::Html);
                    el.append(STATIC_ON_RESPONSE_JS, ContentType::Html);
                    el.append("</script>", ContentType::Html);

                    if let Some(handlers) = el.end_tag_handlers() {
                        handlers.push(Box::new(move |end: &mut EndTag| {
                            end.remove();
                            Ok(())
                        }));
                    }
                    Ok(())
                }),
                element!("html", |el: &mut Element| {
                    if let Some(handlers) = el.end_tag_handlers() {
                        handlers.push(Box::new(|end: &mut EndTag| {
                            end.remove();
                            Ok(())
                        }));
                    }
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        output_sink,
    );

    let mut chuck_buffer = [0u8; CHUCK_BUFFER_SIZE];

    loop {
        let bytes_read = input_file
            .read(&mut chuck_buffer)
            .map_err(|error| Error::ReadInputChunk(input_path.into(), error))?;
        if bytes_read == 0 {
            break;
        }
        rewriter
            .write(&chuck_buffer[..bytes_read])
            .map_err(|error| Error::RewriterWrite(input_path.into(), error))?;
    }

    rewriter
        .end()
        .map_err(|error| Error::RewriterEnd(input_path.into(), error))?;

    output_file
        .flush()
        .map_err(|error| Error::FlushOutputFile(output_path.into(), error))?;

    Ok(task_names)
}
