// use futures_spawn::SpawnHelper;
// use futures_threadpool::ThreadPool;
//
use futures::future::{err, ok, Future};
use futures::stream::Stream;
use hyper::{Body, Error as HyperError};
use http_box::http1::{HttpHandler, Parser, ParserError};
use http_box::fsm::Success;
use std::path::PathBuf;
use std::marker::Send;
use std::io::{Error as IoError, ErrorKind};

///This Enum is used to store a field parsed from a body sent as a multipart form-data
pub enum FormData {
    ///Used when Content-Disposition is NOT a file
    Field {
        ///The field name
        field_name: String,
        ///The unparsed field content
        content: Vec<u8>,
    },
    ///Used when Content-Disposition IS a file
    File {
        ///The field name>
        field_name: String,
        ///The fila name
        file_name: String,
        ///Optional mime type of the transferred file
        content_type: Option<String>,
        ///Path name where the file has been stored on the server
        file_path: PathBuf,
    },
}

pub struct MultipartData {
    fields: Vec<FormData>,
}

impl Default for MultipartData {
    fn default() -> MultipartData {
        MultipartData { fields: Vec::new() }
    }
}

impl MultipartData {
    fn get_fields_iter<'a, 'b: 'a>(
        &'b self,
        name: &'a str,
    ) -> impl Iterator<Item = &'b FormData> + 'a {
        self.fields.iter().filter(move |&field| match *field {
            FormData::Field { ref field_name, .. } | FormData::File { ref field_name, .. } => {
                field_name == name
            }
        })
    }

    pub fn get_fields<'a>(&'a self, name: &str) -> Vec<&'a FormData> {
        self.get_fields_iter(name).collect()
    }

    pub fn get_field<'a>(&'a self, name: &str) -> Option<&'a FormData> {
        self.get_fields_iter(name).next()
    }
}

struct MultipartHandler {}

impl Default for MultipartHandler {
    fn default() -> MultipartHandler {
        MultipartHandler {}
    }
}

impl HttpHandler for MultipartHandler {}

pub struct MultipartParser<'a> {
    handler: MultipartHandler,
    parser: Parser<'a, MultipartHandler>,
}

impl<'a> Default for MultipartParser<'a> {
    fn default() -> MultipartParser<'a> {
        let mut parser = Parser::new();
        parser.init_multipart();
        MultipartParser {
            parser,
            handler: Default::default(),
        }
    }
}

impl<'a> MultipartParser<'a> {
    fn resume(&mut self, stream: &[u8]) -> Result<Success, ParserError> {
        self.parser.resume(&mut self.handler, stream)
    }
}

pub trait Multipart {
    // fn multipart<'a>(&self) -> Box<Future<Item = MultipartData, Error = HyperError> + Send + 'a>;
    // fn multipart(self) -> Box<Future<Item = MultipartData, Error = HyperError> + Send>;
    fn multipart<'b, 'a: 'b>(
        self,
    ) -> Box<Future<Error = HyperError, Item = MultipartParser<'a>> + Send + 'b>;
}

impl Multipart for Body {
    fn multipart<'b, 'a: 'b>(
        self,
    ) -> Box<Future<Error = HyperError, Item = MultipartParser<'a>> + Send + 'b> {
        let p: MultipartParser<'a> = MultipartParser::default();
        Box::new(
            self.fold(p, |mut parser, chunk| match parser.resume(&chunk) {
                Ok(_) => ok(parser),
                Err(parser_error) => err::<_, HyperError>(HyperError::Io(IoError::new(
                    ErrorKind::Other,
                    format!("{:?}", parser_error),
                ))),
            }),
        )
    }
}
