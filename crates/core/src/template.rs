use std::{
    fs,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    result,
};

use lol_html::{
    HtmlRewriter, Settings, element, errors::RewritingError, html_content::ContentType,
};

pub trait Template {
    fn render(self) -> String;
}

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
}

pub type Result<T, E = Error> = result::Result<T, E>;

const CHUCK_BUFFER_SIZE: usize = 16 * 1024;

// TODO: add documentation
#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn parse_and_build<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
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
        // TODO: handle errors
        #[allow(clippy::expect_used)]
        output_file.write_all(c).expect("write chunk to file");
    };

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("title", |el| {
                el.set_inner_content("My new title", ContentType::Text);
                Ok(())
            })],
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
        .map_err(|error| Error::FlushOutputFile(output_path.into(), error))
}
