use std::collections::HashMap;
use std::io::{Read, BufReader, Cursor};
use quick_xml::events::{Event, BytesStart};

const WEB_PREFIX: &[u8] = b"web:";

pub struct Definition {
    /// The UI definition with web-specific extensions removed.
    pub buildable: String,
    /// Map of object id to href target.
    pub hrefs: HashMap<String, String>,
}

impl Definition {
    pub fn new<R: Read>(r: R) -> super::Result<Definition> {
        let mut hrefs = HashMap::new();

        let mut reader = quick_xml::Reader::from_reader(BufReader::new(r));
        let mut writer = quick_xml::Writer::new(Cursor::new(Vec::new()));

        fn attrs_map(bs: &BytesStart) -> super::Result<HashMap<String, String>> {
            let mut attrs = HashMap::new();
            for attr in bs.attributes() {
                let attr = attr?;
                attrs.insert(String::from_utf8(attr.key.to_vec()).unwrap(), String::from_utf8(attr.value.into_owned()).unwrap());
            }
            Ok(attrs)
        }

        let mut trim_byte_start = |bs: &BytesStart| {
            let attrs = attrs_map(bs)?;
            let mut result = BytesStart::owned_name(bs.name());
            for attr in bs.attributes() {
                let attr = attr?;
                if attr.key.starts_with(WEB_PREFIX) {
                    let value = String::from_utf8(attr.value.to_vec())?;
                    match &attr.key[WEB_PREFIX.len()..] {
                        b"href" => match attrs.get("id") {
                            Some(id) => { hrefs.insert(id.clone(), value); },
                            None => return Err(crate::error::Error::MissingRequiredAttribute("id")),
                        },
                        k => println!("unknown web attribute: {}", String::from_utf8(k.to_vec())?),
                    }
                } else {
                    result.push_attribute(attr);
                }
            }
            Ok(result)
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event(&mut buf)? {
                Event::Eof => break,
                Event::Start(ref bs) => writer.write_event(Event::Start(trim_byte_start(bs)?))?,
                Event::Empty(ref bs) => writer.write_event(Event::Empty(trim_byte_start(bs)?))?,
                e => writer.write_event(&e)?,
            }
        }

        Ok(Definition{
            buildable: String::from_utf8(writer.into_inner().into_inner())?,
            hrefs,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_new_definition_removes_web_attrs() -> crate::Result<()> {
        let body = r#"<interface><object id="button" web:clicked="do_something();" /></interface>"#;
        let def = Definition::new(body.as_bytes())?;
        assert_eq!(def.buildable, r#"<interface><object id="button"/></interface>"#);
        Ok(())
    }

    #[test]
    pub fn test_parse_href() -> crate::Result<()> {
        let body = r#"<interface><object id="button" web:href="/some/page" /></interface>"#;
        let def = Definition::new(body.as_bytes())?;
        assert_eq!(def.hrefs, HashMap::from([(String::from("button"), String::from("/some/page"))]));
        Ok(())
    }
}
