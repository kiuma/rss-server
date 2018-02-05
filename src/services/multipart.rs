// use futures_spawn::SpawnHelper;
// use futures_threadpool::ThreadPool;
//
use futures::future::{err, ok, Future};
use futures::stream::Stream;
use hyper::{Body, Error as HyperError};
use http_box::http1::{HttpHandler, Parser, ParserError, State};
use http_box::util::FieldIterator;
use http_box::fsm::Success;
use std::path::PathBuf;
use std::marker::Send;
use std::io::{Error as IoError, ErrorKind};
use std::error::Error;
use std::fmt;
use std::string::FromUtf8Error;

#[derive(Debug)]
struct MultipartError {
    message: String,
}

impl MultipartError {
    fn new(message: &str) -> MultipartError {
        MultipartError {
            message: String::from(message),
        }
    }
}

impl fmt::Display for MultipartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MultipartError {
    fn description(&self) -> &str {
        self.message.as_str()
    }
}

impl From<FromUtf8Error> for MultipartError {
    fn from(e: FromUtf8Error) -> Self {
        Self {
            message: format!("{}", e),
        }
    }
}

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

struct MultipartHandler {
    error: Option<MultipartError>,
    count: usize,
    data: Vec<u8>,
    curr_formdata: Option<FormData>,
    pub multipart_data: MultipartData,
    name_buf: Vec<u8>,
    state: State,
    value_buf: Vec<u8>,
}

impl<'a> Default for MultipartHandler {
    fn default() -> MultipartHandler {
        MultipartHandler {
            error: None,
            count: 0,
            data: Vec::new(),
            curr_formdata: None,
            multipart_data: Default::default(),
            name_buf: Vec::new(),
            state: State::None,
            value_buf: Vec::new(),
        }
    }
}

impl MultipartHandler {
    fn flush_header(&mut self) -> Result<(), MultipartError> {
        if self.name_buf.len() > 0 && self.value_buf.len() > 0 {
            let header_name = match String::from_utf8(self.name_buf) {
                Ok(val) => val,
                Err(e) => return Err(MultipartError::from(e)),
            };
            let header_value = match String::from_utf8(self.value_buf) {
                Ok(val) => val,
                Err(e) => return Err(MultipartError::from(e)),
            };

            // if header_name.to_lowercase() == "content-type" {
            //     match self.curr_formdata {
            //         Some(fromdata) => {
            //             self.curr_formdata:
            //         }
            //     }
            // } else {
            //     match self.curr_formdata {
            //         None => return MultipartError::new(format!("Body field didn't start with Content-Disposition {}", header_name), None);
            //         Some(formdata) => formdata.
            //     }
            // }
        }

        self.name_buf = Vec::new();
        self.value_buf = Vec::new();
        Ok(())
    }
}

impl HttpHandler for MultipartHandler {
    fn on_header_name(&mut self, name: &[u8]) -> bool {
        if self.state == State::HeaderValue {
            self.flush_header();
        }

        self.name_buf.extend_from_slice(name);

        self.state = State::HeaderName;
        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        self.value_buf.extend_from_slice(value);

        self.state = State::HeaderValue;
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.flush_header();

        true
    }

    fn on_multipart_begin(&mut self) -> bool {
        self.count += 1;

        if self.count > 1 {
            // we found a new piece of data, and it's not the first one, so force an exit
            // so we can compare
            false
        } else {
            // first piece of data, continue as normal
            true
        }
    }

    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        self.data.extend_from_slice(data);

        true
    }
}

pub struct MultipartParser<'a> {
    handler: MultipartHandler,
    parser: Parser<'a, MultipartHandler>,
}

impl<'a> MultipartParser<'a> {
    fn new(boundary: &'a [u8]) -> MultipartParser<'a> {
        let mut parser = Parser::new();
        parser.init_multipart();
        parser.set_boundary(boundary);
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
    fn multipart<'b, 'a: 'b>(
        self,
        boundary: &'a [u8],
    ) -> Box<Future<Error = HyperError, Item = MultipartParser<'a>> + 'b>;
}

impl Multipart for Body {
    fn multipart<'b, 'a: 'b>(
        self,
        boundary: &'a [u8],
    ) -> Box<Future<Error = HyperError, Item = MultipartParser<'a>> + 'b> {
        let p: MultipartParser<'a> = MultipartParser::new(boundary);
        // p.set_boundary(b.as_ref().unwrap().as_bytes());
        Box::new(self.fold(
            p,
            |mut parser: MultipartParser<'a>, chunk| match parser.resume(&chunk) {
                Ok(_) => ok(parser),
                Err(parser_error) => err(HyperError::Io(IoError::new(
                    ErrorKind::Other,
                    format!("{:?}", parser_error),
                ))),
            },
        ))
    }
}

// impl Multipart for Body {
//     fn multipart<'b, 'a: 'b>(
//         self,
//         boundary: &'a [u8],
//     ) -> Box<Future<Error = HyperError, Item = MultipartParser<'a>> + Send + 'b> {
//         let p: MultipartParser<'a> = MultipartParser::new(boundary);
//         // p.set_boundary(b.as_ref().unwrap().as_bytes());
//         Box::new(
//             self.fold(p, |mut parser, chunk| match parser.resume(&chunk) {
//                 Ok(_) => ok(parser),
//                 Err(parser_error) => err::<_, HyperError>(HyperError::Io(IoError::new(
//                     ErrorKind::Other,
//                     format!("{:?}", parser_error),
//                 ))),
//             }),
//         )
//     }
// }

//========================== TESTS =====================================================//
#[cfg(test)]
mod tests {}
