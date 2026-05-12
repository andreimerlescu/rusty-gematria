// update App struct to hold tagged_words and delay
pub struct App {
    mode:          Mode,
    active_pane:   Pane,
    input:         String,
    phrases:       Vec<textee::Phrase>,
    results:       Vec<cipher::Gematria>,
    results_state: TableState,
    matches:       Vec<String>,
    matrix:        Matrix,
    tagged_words:  Vec<phrase::TaggedWord>,
    limit:         usize,
    delay:         u64,
    should_quit:   bool,
    last_key_g:    bool,
}

impl App {
    pub fn new(
        matrix:       Matrix,
        tagged_words: Vec<phrase::TaggedWord>,
        limit:        usize,
        delay:        u64,
        initial_text: Option<String>,
    ) -> App {
        let mut app = App {
            mode:          Mode::Normal,
            active_pane:   Pane::Input,
            input:         initial_text.unwrap_or_default(),
            phrases:       Vec::new(),
            results:       Vec::new(),
            results_state: TableState::default(),
            matches:       Vec::new(),
            matrix,
            tagged_words,
            limit,
            delay,
            should_quit:   false,
            last_key_g:    false,
        };
        // if we have initial text compute results immediately
        if !app.input.is_empty() {
            app.recompute();
        }
        app
    }
}
