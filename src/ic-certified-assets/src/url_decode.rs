use std::fmt;

/// An iterator-like structure that decode a URL.
struct UrlDecode<'a> {
    bytes: std::slice::Iter<'a, u8>,
}

fn convert_percent(iter: &mut std::slice::Iter<u8>) -> Option<u8> {
    let mut cloned_iter = iter.clone();
    let result = match cloned_iter.next()? {
        b'%' => b'%',
        h => {
            let h = char::from(*h).to_digit(16)?;
            let l = char::from(*cloned_iter.next()?).to_digit(16)?;
            h as u8 * 0x10 + l as u8
        }
    };
    // Update this if we make it this far, otherwise "reset" the iterator.
    *iter = cloned_iter;
    Some(result)
}

#[derive(Debug, PartialEq)]
pub enum UrlDecodeError {
    InvalidPercentEncoding,
}

impl fmt::Display for UrlDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPercentEncoding => write!(f, "invalid percent encoding"),
        }
    }
}

impl<'a> Iterator for UrlDecode<'a> {
    type Item = Result<char, UrlDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let b = self.bytes.next()?;
        match b {
            b'%' => Some(
                convert_percent(&mut self.bytes)
                    .map(char::from)
                    .ok_or(UrlDecodeError::InvalidPercentEncoding),
            ),
            b'+' => Some(Ok(' ')),
            x => Some(Ok(char::from(*x))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let bytes = self.bytes.len();
        (bytes / 3, Some(bytes))
    }
}

pub fn url_decode(url: &str) -> Result<String, UrlDecodeError> {
    UrlDecode {
        bytes: url.as_bytes().iter(),
    }
    .collect()
}
