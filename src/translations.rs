pub enum Message {
    Hello,
    Scanning,
    ConnectedPeers,

    ConnectMorePeers,
    Confirm,
    Cancel,
    Stop,
}

impl firefly_ui::Translate<'static> for Message {
    fn translate_english(&self) -> &'static str {
        match self {
            Message::Hello => "hello, ",
            Message::Scanning => "scanning...",
            Message::ConnectedPeers => "connected peers",

            Message::ConnectMorePeers => "connect more peers",
            Message::Confirm => "confirm",
            Message::Cancel => "cancel",
            Message::Stop => "stop",
        }
    }

    fn translate_dutch(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_french(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_german(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_italian(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_polish(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_romanian(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_russian(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_spanish(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_swedish(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_turkish(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_ukrainian(&self) -> &'static str {
        self.translate_english()
    }

    fn translate_toki_pona(&self) -> &'static str {
        self.translate_english()
    }
}
