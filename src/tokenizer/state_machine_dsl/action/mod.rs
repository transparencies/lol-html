#[macro_use]
mod helpers;

#[macro_use]
mod state_transition;

macro_rules! action {

    // Token emission
    //--------------------------------------------------------------------
    ( | $self:tt, $ch:ident |> emit_eof ) => {
        action_helper!(@emit_lex_result |$self|> ShallowToken::Eof, None);
        $self.finished = true;
    };

    ( | $self:tt, $ch:ident |> emit_chars ) => {
        if $self.pos > $self.raw_start {
            // NOTE: unlike any other tokens, character tokens don't have
            // any lexical symbols that determine their bounds. Therefore,
            // representation of character token content is the raw slice.
            // Also, we always emit characters if we encounter some other bounded
            // lexical structure and, thus, we use exclusive range for the raw slice.
            action_helper!(@emit_lex_result_with_raw_exclusive |$self|> ShallowToken::Character);
        }
    };

    ( | $self:tt, $ch:ident |> emit_current_token ) => {
        match $self.current_token.take() {
            Some(token) => {
                action_helper!(@emit_lex_result_with_raw_inclusive |$self|> token);
            }
            None => unreachable!("Current token should exist at this point")
        }
    };


    // Slices
    //--------------------------------------------------------------------
    ( | $self:tt, $ch:ident |> start_raw ) => {
        $self.raw_start = $self.pos;
    };

    ( | $self:tt, $ch:ident |> start_token_part ) => {
        $self.token_part_start = $self.pos - $self.raw_start;
    };


    // Token creation
    //--------------------------------------------------------------------
    ( | $self:tt, $ch:ident |> create_start_tag ) => {
        $self.attr_buffer.borrow_mut().clear();

        $self.current_token = Some(ShallowToken::StartTag {
            name: SliceRange::default(),
            name_hash: Some(0),
            attributes: Rc::clone(&$self.attr_buffer),
            self_closing: false,
        });
    };

    ( | $self:tt, $ch:ident |> create_end_tag ) => {
        $self.current_token = Some(ShallowToken::EndTag {
            name: SliceRange::default(),
            name_hash: Some(0),
        });
    };

    ( | $self:tt, $ch:ident |> create_doctype ) => {
        $self.current_token = Some(ShallowToken::Doctype {
            name: None,
            public_id: None,
            system_id: None,
            force_quirks: false,
        });
    };

    ( | $self:tt, $ch:ident |> create_comment ) => {
        $self.current_token = Some(ShallowToken::Comment(SliceRange::default()));
    };


    // Comment parts
    //--------------------------------------------------------------------
    ( | $self:tt, $ch:ident |> mark_comment_text_end ) => {
        if let Some(ShallowToken::Comment(ref mut text)) = $self.current_token {
            action_helper!(@set_token_part_range |$self|> text);
        }
    };

    ( | $self:tt, $ch:ident |> shift_comment_text_end_by $shift:expr ) => {
        if let Some(ShallowToken::Comment(ref mut text)) = $self.current_token {
            text.end += $shift;
        }
    };


    // Doctype parts
    //--------------------------------------------------------------------
    ( | $self:tt, $ch:ident |> set_force_quirks ) => {
        if let Some(ShallowToken::Doctype { ref mut force_quirks, .. }) = $self.current_token {
            *force_quirks = true;
        }
    };

    ( | $self:tt, $ch:ident |> finish_doctype_name ) => {
        if let Some(ShallowToken::Doctype { ref mut name, .. }) = $self.current_token {
            action_helper!(@set_opt_token_part_range |$self|> name);
        }
    };

    ( | $self:tt, $ch:ident |> finish_doctype_public_id ) => {
        if let Some(ShallowToken::Doctype { ref mut public_id, .. }) = $self.current_token {
            action_helper!(@set_opt_token_part_range |$self|> public_id);
        }
    };

    ( | $self:tt, $ch:ident |> finish_doctype_system_id ) => {
        if let Some(ShallowToken::Doctype { ref mut system_id, .. }) = $self.current_token {
            action_helper!(@set_opt_token_part_range |$self|> system_id);
        }
    };


    // Tag parts
    //--------------------------------------------------------------------
    ( | $self:tt, $ch:ident |> finish_tag_name ) => {
        action_helper!(@update_tag_part |$self|> name, {
            action_helper!(@set_token_part_range |$self|> name);
        });
    };

    ( | $self:tt, $ch:ident |> update_tag_name_hash ) => {
        if let Some(ch) = $ch {
            action_helper!(@update_tag_part |$self|> name_hash, {
                *name_hash = update_tag_name_hash(*name_hash, ch);
            });
        }
    };

    ( | $self:tt, $ch:ident |> mark_as_self_closing ) => {
        if let Some(ShallowToken::StartTag { ref mut self_closing, .. }) = $self.current_token {
            *self_closing = true;
        }
    };


    // Attributes
    //--------------------------------------------------------------------
    ( | $self:tt, $ch:ident |> start_attr ) => {
        // NOTE: create attribute only if we are parsing a start tag
        if let Some(ShallowToken::StartTag {..}) = $self.current_token {
            $self.current_attr = Some(ShallowAttribute::default());
            action!(|$self, $ch|> start_token_part);
        }
    };

    ( | $self:tt, $ch:ident |> finish_attr_name ) => {
        action_helper!(@finish_attr_part |$self|> name);
    };

    ( | $self:tt, $ch:ident |> finish_attr_value ) => {
        action_helper!(@finish_attr_part |$self|> value);
    };

    ( | $self:tt, $ch:ident |> finish_attr ) => {
        match $self.current_attr.take() {
            Some(attr) => {
                $self.attr_buffer.borrow_mut().push(attr);
            }
            // NOTE: end tag case
            None => ()
        }
    };


    // Quotes
    //--------------------------------------------------------------------
    ( | $self:tt, $ch:ident |> set_closing_quote_to_double ) => {
        $self.closing_quote = b'"';
    };

    ( | $self:tt, $ch:ident |> set_closing_quote_to_single ) => {
        $self.closing_quote = b'\'';
    };

}