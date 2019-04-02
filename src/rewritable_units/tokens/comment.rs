use super::{OrderingMutations, Token};
use crate::base::Bytes;
use encoding_rs::Encoding;
use std::fmt::{self, Debug};

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum CommentTextError {
    #[fail(display = "Comment text shouldn't contain comment closing sequence (`-->`).")]
    CommentClosingSequence,
    #[fail(display = "Comment text contains a character that can't \
                      be represented in the document's character encoding.")]
    UnencodableCharacter,
}

pub struct Comment<'i> {
    text: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
    ordering_mutations: OrderingMutations,
}

impl<'i> Comment<'i> {
    pub(super) fn new_token(
        text: Bytes<'i>,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Token<'i> {
        Token::Comment(Comment {
            text,
            raw: Some(raw),
            encoding,
            ordering_mutations: OrderingMutations::default(),
        })
    }

    #[inline]
    pub fn text(&self) -> String {
        self.text.as_string(self.encoding)
    }

    #[inline]
    pub fn set_text(&mut self, text: &str) -> Result<(), CommentTextError> {
        if text.find("-->").is_some() {
            Err(CommentTextError::CommentClosingSequence)
        } else {
            // NOTE: if character can't be represented in the given
            // encoding then encoding_rs replaces it with a numeric
            // character reference. Character references are not
            // supported in comments, so we need to bail.
            match Bytes::from_str_without_replacements(text, self.encoding) {
                Some(text) => {
                    self.text = text.into_owned();
                    self.raw = None;

                    Ok(())
                }
                None => Err(CommentTextError::UnencodableCharacter),
            }
        }
    }

    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        self.raw.as_ref()
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        output_handler(b"<!--");
        output_handler(&self.text);
        output_handler(b"-->");
    }
}

impl_common_token_api!(Comment);

impl Debug for Comment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Comment")
            .field("text", &self.text())
            .finish()
    }
}