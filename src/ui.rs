use std::collections::HashMap;
use std::io::{Read, BufReader, Cursor};
use quick_xml::events::{Event, BytesStart};
use quick_xml::name::QName;

const PREFIX: &[u8] = b"web";
const SCRIPT_TAG: &[u8] = b"script";
const STYLE_TAG: &[u8] = b"style";
const PAGE_TAG: &[u8] = b"page";

pub struct Definition {
    /// The UI definition with web-specific extensions removed.
    pub buildable: String,
    /// Map of object id to href target.
    pub hrefs: HashMap<String, String>,
    /// List of scripts to execute.
    pub scripts: Vec<crate::script::Script>,
    // Custom styles
    pub styles: String,
    /// Title of the page.
    pub title: Option<String>,
}

impl Definition {
    pub fn new<R: Read>(r: R) -> super::Result<Definition> {
        let mut hrefs = HashMap::new();
        let mut scripts = Vec::new();
        let mut styles = String::new();
        let mut title = None;

        let mut reader = quick_xml::Reader::from_reader(BufReader::new(r));
        let mut writer = quick_xml::Writer::new(Cursor::new(Vec::new()));

        fn attrs_map(bs: &BytesStart) -> super::Result<HashMap<String, String>> {
            let mut attrs = HashMap::new();
            for attr in bs.attributes() {
                let attr = attr?;
                attrs.insert(String::from_utf8(attr.key.0.to_vec()).unwrap(), String::from_utf8(attr.value.into_owned()).unwrap());
            }
            Ok(attrs)
        }

        let mut id_autogenerator = IdAutogenerator::new();

        let mut trim_bytes_start = |bs: &BytesStart| -> crate::Result<BytesStart> {
            let attrs = attrs_map(bs)?;
            let mut result = bs.to_owned();
            for attr in bs.attributes() {
                let attr = attr?;
                match parse_web_tag(&attr.key) {
                    Some(web_tag) => {
                        let value = String::from_utf8(attr.value.to_vec())?;
                        match web_tag {
                            b"href" => {
                                let id = match attrs.get("id") {
                                    Some(id) => id.clone(),
                                    None => {
                                        let class = attrs.get("class").expect("expected 'class' attribute to be present");
                                        let id = id_autogenerator.next(&class);
                                        result.push_attribute(("id", id.as_str()));
                                        id
                                    },
                                };
                                hrefs.insert(id.to_string(), value);
                            },
                            k => println!("unknown web attribute: {}", String::from_utf8(k.to_vec())?),
                        }
                    },
                    None => result.push_attribute(attr),
                }
            }
            Ok(result)
        };

        let mut buf = Vec::new();

        let mut reading_script = false;
        let mut current_script_type = None;
        let mut current_script = String::new();

        let mut reading_style = false;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Eof => break,
                Event::Start(ref bs) => match parse_web_tag(&bs.name()) {
                    Some(SCRIPT_TAG) => {
                        let attrs = attrs_map(bs)?;
                        match attrs.get("type") {
                            None => println!("script tag found, but no type was specified"),
                            Some(r#type) => match crate::script::Lang::from(r#type) {
                                None => println!("script tag found with unknown type '{}'", r#type),
                                Some(lang) => {
                                    current_script_type = Some(lang);
                                    current_script = String::new();
                                    reading_script = true;
                                },
                            },
                        }
                    },
                    Some(STYLE_TAG) => {
                        reading_style = true;
                    },
                    _ => writer.write_event(Event::Start(trim_bytes_start(bs)?))?,
                },
                Event::Text(bt) => {
                    if reading_script {
                        current_script.push_str(&mut bt.unescape()?);
                    } else if reading_style {
                        styles.push_str(&mut bt.unescape()?);
                    } else {
                        writer.write_event(Event::Text(bt))?;
                    }
                },
                Event::End(be) => match parse_web_tag(&be.name()) {
                    Some(SCRIPT_TAG) => {
                        if reading_script {
                            scripts.push(crate::script::Script::new(current_script_type.unwrap(), current_script.clone()));
                            reading_script = false;
                        }
                    },
                    Some(STYLE_TAG) => {
                        reading_style = false;
                    },
                    _ => writer.write_event(Event::End(be))?,
                },
                Event::Empty(ref bs) => match parse_web_tag(&bs.name()) {
                    Some(PAGE_TAG) => {
                        let attrs = attrs_map(bs)?;
                        if let Some(v) = attrs.get("title") {
                            title = Some(v.clone());
                        }
                    },
                    _ => writer.write_event(Event::Empty(trim_bytes_start(bs)?))?,
                },
                e => writer.write_event(&e)?,
            }
        }

        Ok(Definition{
            buildable: String::from_utf8(writer.into_inner().into_inner())?,
            hrefs,
            scripts,
            styles,
            title,
        })
    }
}

fn parse_web_tag<'a>(name: &'a QName) -> Option<&'a [u8]> {
    match name.prefix() {
        Some(ref prefix) if prefix.as_ref() == PREFIX => Some(name.local_name().into_inner()),
        _ => None,
    }
}

struct IdAutogenerator(HashMap<String, i8>);

impl IdAutogenerator {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn next(&mut self, class: &String) -> String {
        let n = self.0.entry(class.to_string()).or_insert(0);
        *n += 1;
        format!("{}-{}", class, *n)
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

    #[test]
    pub fn test_autogen_ids() {
        let mut id_autogenerator = IdAutogenerator::new();
        assert_eq!(id_autogenerator.next(&"GtkButton".to_string()), "GtkButton-1");
        assert_eq!(id_autogenerator.next(&"GtkButton".to_string()), "GtkButton-2");
        assert_eq!(id_autogenerator.next(&"GtkButton".to_string()), "GtkButton-3");
        assert_eq!(id_autogenerator.next(&"GtkLabel".to_string()), "GtkLabel-1");
    }

    #[test]
    pub fn test_parse_web_tag() {
        assert_eq!(parse_web_tag(&QName(b"web:script")), Some(b"script" as &[u8]));
        assert_eq!(parse_web_tag(&QName(b"web:page")), Some(b"page" as &[u8]));
        assert_eq!(parse_web_tag(&QName(b"object")), None);
    }
}
