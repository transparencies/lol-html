define_state_group!(rawtext_states_group = {

    pub rawtext_state <-- ( notify_text_parsing_mode_change TextParsingMode::RawText; ) {
        b'<' => ( emit_chars; --> rawtext_less_than_sign_state )
        eoc  => ( emit_chars; )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    rawtext_less_than_sign_state {
        b'/' => ( --> rawtext_end_tag_open_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ( emit_chars; reconsume in rawtext_state )
    }

    rawtext_end_tag_open_state {
        alpha => ( create_end_tag; start_token_part; update_tag_name_hash; --> rawtext_end_tag_name_state )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_chars; reconsume in rawtext_state )
    }

    rawtext_end_tag_name_state {
        whitespace => (
            if appropriate_end_tag ( finish_tag_name; --> before_attribute_name_state )
            else ( emit_chars; reconsume in rawtext_state )
        )

        b'/' => (
            if appropriate_end_tag ( finish_tag_name; --> self_closing_start_tag_state )
            else ( emit_chars; reconsume in rawtext_state )
        )

        b'>' => (
            if appropriate_end_tag ( finish_tag_name; emit_current_token; --> data_state )
            else ( emit_chars; reconsume in rawtext_state )
        )

        alpha => ( update_tag_name_hash; )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_chars; reconsume in rawtext_state )
    }

});
