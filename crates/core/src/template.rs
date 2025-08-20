use std::{
    collections::BTreeSet,
    fs,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    result,
    sync::mpsc,
};

use lol_html::{
    HtmlRewriter, OutputSink, Settings, element,
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
    #[error("failed to write output chunk: {0}: {1}")]
    WriteOutputChunk(PathBuf, #[source] RewritingError),
    #[error("failed to write output file chunk: {0}: {1}")]
    WriteOutputFileChunk(PathBuf, #[source] io::Error),
    #[error("failed to rewrite the template: {1}")]
    RewriteTemplate(PathBuf, #[source] RewritingError),
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
    #[error(r#"failed to include fragment '{tag}[data-htms="{path}"]' at byte offset {offset}: {source}"#)]
    IncludeFragment {
        tag: String,
        path: PathBuf,
        offset: usize,
        source: io::Error,
    },
}

pub type Result<T, E = Error> = result::Result<T, E>;

pub type TaskNames = BTreeSet<String>;

#[derive(Debug, Default)]
pub struct Build {
    has_html_tag: bool,
    task_names: BTreeSet<String>,
}

impl Build {
    #[inline]
    #[must_use]
    pub const fn has_html_tag(&self) -> bool {
        self.has_html_tag
    }

    #[inline]
    #[must_use]
    pub const fn task_names(&self) -> &TaskNames {
        &self.task_names
    }
}

// TODO: add documentation
#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn parse_and_build<I: AsRef<Path>, A: AsRef<Path>>(
    input_path: I,
    output_path: A,
) -> Result<Build> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    let mut input_file =
        File::open(input_path).map_err(|error| Error::OpenInputPath(input_path.into(), error))?;

    if let Some(output_directory) = output_path.parent() {
        fs::create_dir_all(output_directory)
            .map_err(|error| Error::CreateOutputDirectory(output_directory.into(), error))?;
    }

    let mut output_file = File::create(output_path)
        .map_err(|error| Error::CreateOutputFile(output_path.into(), error))?;

    let mut build = Build::default();
    let (error_tx, error_rx) = mpsc::channel();

    {
        let dynamic_rewriter_sink = |c: &[u8]| {
            if let Err(error) = output_file.write_all(c) {
                #[allow(clippy::expect_used)]
                error_tx
                    .send(Error::WriteOutputFileChunk(output_path.into(), error))
                    .expect("send dynamic sink error");
            }
        };

        let mut dynamic_rewriter = make_dynamic_rewriter(&mut build, dynamic_rewriter_sink);

        let static_rewriter_sink = |c: &[u8]| {
            if let Err(error) = dynamic_rewriter.write(c) {
                #[allow(clippy::expect_used)]
                error_tx
                    .send(Error::WriteOutputChunk(output_path.into(), error))
                    .expect("send static sink error");
            }
        };

        let mut static_rewriter = make_static_rewriter(input_path, static_rewriter_sink);

        let mut chuck_buffer = [0u8; CHUCK_BUFFER_SIZE];

        loop {
            if let Ok(error) = error_rx.try_recv() {
                return Err(error);
            }

            let bytes_read = input_file
                .read(&mut chuck_buffer)
                .map_err(|error| Error::ReadInputChunk(input_path.into(), error))?;

            if bytes_read == 0 {
                break;
            }

            static_rewriter
                .write(&chuck_buffer[..bytes_read])
                .map_err(|error| Error::RewriteTemplate(input_path.into(), error))?;
        }
    }

    output_file
        .flush()
        .map_err(|error| Error::FlushOutputFile(output_path.into(), error))?;

    Ok(build)
}

fn make_dynamic_rewriter<O: OutputSink>(
    build: &'_ mut Build,
    rewriter_sink: O,
) -> HtmlRewriter<'_, O> {
    HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("html", |el: &mut Element| {
                    build.has_html_tag = true;

                    if let Some(handlers) = el.end_tag_handlers() {
                        handlers.push(Box::new(|end: &mut EndTag| {
                            end.remove();
                            Ok(())
                        }));
                    }
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
                    build.task_names.insert(method_name.to_string());

                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        rewriter_sink,
    )
}

fn make_static_rewriter<O: OutputSink>(
    input_path: &'_ Path,
    static_rewriter_sink: O,
) -> HtmlRewriter<'_, O> {
    HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!(r#"[data-htms^="include:"]"#, |el| {
                let attribute_value = el.get_attribute("data-htms").unwrap_or_default();
                let (_, file_path) = attribute_value.trim().split_once(':').unwrap_or_default();

                let include_path = Path::new(input_path).with_file_name(file_path);
                let html =
                    fs::read_to_string(include_path).map_err(|source| Error::IncludeFragment {
                        tag: el.tag_name(),
                        path: file_path.into(),
                        offset: el.source_location().bytes().start,
                        source,
                    })?;

                el.replace(html.as_str(), ContentType::Html);

                Ok(())
            })],
            ..Settings::default()
        },
        static_rewriter_sink,
    )
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod parse_and_build {
    use std::{
        fs,
        io::Write,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{Build, STATIC_ON_RESPONSE_JS, STATIC_STYLE_CSS, parse_and_build};
    use crate::template;

    fn unique_path(prefix: &str, extension: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("get nanos since epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}_{nanos}.{extension}"))
    }

    fn write_temp_file(prefix: &str, contents: &str) -> PathBuf {
        let path = unique_path(prefix, "html");

        let mut file = fs::File::create(&path).expect("create temp file");

        file.write_all(contents.as_bytes())
            .expect("write temp file");

        path
    }

    fn write_input_file(contents: &str) -> PathBuf {
        write_temp_file("htms_test_input", contents)
    }

    fn read_output_string(path: &PathBuf) -> String {
        fs::read_to_string(path).expect("read output file")
    }

    fn temp_build(input_html: &str) -> (PathBuf, template::Result<Build>) {
        let input_path = write_input_file(input_html);
        let output_path = unique_path("htms_test_output", "html");
        let build = parse_and_build(&input_path, &output_path);

        (output_path, build)
    }

    fn temp_build_with_rendered(input_html: &str) -> (Build, String) {
        let (output_path, build) = temp_build(input_html);
        let rendered = read_output_string(&output_path);

        (build.expect("temp_build"), rendered)
    }

    #[test]
    fn without_html_tag_does_not_inject_style_and_script() {
        let (build, rendered) = temp_build_with_rendered("<p>Content</p>");

        assert!(!build.has_html_tag());
        assert!(!rendered.contains("<style>"));
        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("<p>Content</p>"));
    }

    #[test]
    fn with_html_tag_injects_style_and_script_and_removes_end_tags() {
        let (build, rendered) = temp_build_with_rendered(
            r"<!doctype html><html><head></head><body>
            <p>Content</p>
            </body></html>",
        );

        assert!(build.has_html_tag());
        assert!(rendered.contains(format!("<style>{STATIC_STYLE_CSS}</style>").as_str()));
        assert!(rendered.contains(format!("<script>{STATIC_ON_RESPONSE_JS}</script>").as_str()));
        assert!(!rendered.contains("</body>"));
        assert!(!rendered.contains("</html>"));
    }

    #[test]
    fn collects_task_names_and_normalizes_attribute() {
        let (build, rendered) = temp_build_with_rendered(
            r#"<!doctype html><html><head></head><body>
            <div data-htms="fn:news"></div>
            <div data-htms="fn:blog_posts"></div>
            </body></html>"#,
        );
        let task_names = build.task_names();

        assert!(task_names.contains("news"));
        assert!(task_names.contains("blog_posts"));
        assert!(rendered.contains(r#"data-htms="news""#));
        assert!(rendered.contains(r#"data-htms="blog_posts""#));
        assert!(!rendered.contains(r#"data-htms="fn:news""#));
        assert!(!rendered.contains(r#"data-htms="fn:blog_posts""#));
    }

    #[test]
    fn include_fragment_and_collects_task_names() {
        let include_path = write_temp_file(
            "included",
            r#"
        <strong>Some HTML to be included in tests.</strong>
        <div data-htms="fn:included_fn"></div>
        "#,
        );
        let include_file_name = include_path.file_name().unwrap();
        let input_path = write_input_file(
            format!(
                r#"<!doctype html><html><head></head><body>
            <div data-htms="fn:news"></div>
            <div data-htms="include:{}"></div>
            </body></html>"#,
                include_file_name.display()
            )
            .as_str(),
        );
        let output_path = unique_path("htms_test_output", "html");
        let build = parse_and_build(&input_path, &output_path).expect("parse and build");
        let rendered = read_output_string(&output_path);
        let task_names = build.task_names();

        assert!(task_names.contains("news"));
        assert!(task_names.contains("included_fn"));
        assert!(rendered.contains("<strong>Some HTML to be included in tests.</strong>"));
        assert!(rendered.contains(r#"data-htms="news""#));
        assert!(rendered.contains(r#"data-htms="included_fn""#));
        assert!(!rendered.contains(r#"data-htms="fn:news""#));
        assert!(!rendered.contains(r#"data-htms="fn:included_fn""#));
    }

    #[test]
    fn fails_on_invalid_htms_attribute() {
        let (_, build) = temp_build(
            r#"<!doctype html><html><head></head><body>
            <div data-htms="fn:invalid-name"></div>
            </body></html>"#,
        );
        let is_error = build.is_err();
        let message = build.unwrap_err().to_string();

        assert!(is_error);
        assert!(message.contains("failed to write output chunk: "));
        assert!(message.contains("invalid attribute 'div[data-htms]' at byte offset 53"));
    }

    #[test]
    fn fails_on_input_file_not_found() {
        let input_path = unique_path("htms_test_input", "html");
        let output_path = unique_path("htms_test_output", "html");
        let build = parse_and_build(&input_path, &output_path);

        let is_error = build.is_err();
        let message = build.unwrap_err().to_string();

        assert!(is_error);
        assert!(message.contains("failed to open input path: "));
    }
}
