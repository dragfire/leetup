pub enum Icon {
    Yes,
    _No,
    Star,
    _Unstar,
    Lock,
    Empty,
}

impl ToString for Icon {
    fn to_string(&self) -> String {
        match self {
            Icon::Yes => "✔".to_string(),
            Icon::_No => "✘".to_string(),
            Icon::Star => "★".to_string(),
            Icon::_Unstar => "☆".to_string(),
            Icon::Lock => "🔒".to_string(),
            Icon::Empty => " ".to_string(),
        }
    }
}
