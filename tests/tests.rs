#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use magneturi::MagnetUri;

    #[test]
    fn parse_btih() {
        let uri = MagnetUri::from_str("magnet:?xt=urn:btih:99ab87be389e5487ff626162a5a5988ce696574a&dn=Name&tr=http%3A%2F%example.tracker.com%3A7777%2Fannounce");
        
        assert!(uri.is_ok())
    }
}
