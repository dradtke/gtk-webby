use std::io::{Read, BufReader, Cursor};
use quick_xml::events::{Event, BytesStart};

pub fn remove_web_attrs<R: Read>(r: R) -> super::Result<Vec<u8>> {
    let mut reader = quick_xml::Reader::from_reader(BufReader::new(r));
    let mut writer = quick_xml::Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf)? {
            Event::Eof => break,
            /*
            Event::Start(bs) => {
                let elem = BytesStart::owned_name(bs.name());
            },
            */
            e => writer.write(&e)?,
        }
    }

    Ok(writer.into_inner().into_inner())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_remove_web_attrs() -> crate::Result<()> {
        let body = r#"<interface><object id="button" web:clicked="do_something();" /></interface>"#;
        let result = remove_web_attrs(body.as_bytes())?;
        let string_result = String::from_utf8(result).unwrap();
        assert_eq!(string_result, body);
        Ok(())
    }
}
