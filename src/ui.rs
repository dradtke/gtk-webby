use std::collections::HashMap;
use std::io::{Read, BufReader, Cursor};
use quick_xml::events::{Event, BytesStart};

const WEB_PREFIX: &[u8] = b"web:";

pub struct Definition {
    /// The UI definition with web-specific extensions removed.
    pub buildable: String,
    /// Map of object id to href target.
    pub hrefs: HashMap<String, String>,
    /// List of scripts to execute.
    pub scripts: Vec<(crate::script::Lang, String)>,
}

impl Definition {
    pub fn new<R: Read>(r: R) -> super::Result<Definition> {
        let mut hrefs = HashMap::new();
        let mut scripts = Vec::new();

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

        let mut id_autogenerator = IdAutogenerator::new();

        let mut trim_bytes_start = |bs: &BytesStart| -> crate::Result<BytesStart> {
            let attrs = attrs_map(bs)?;
            let mut result = BytesStart::owned_name(bs.name());
            for attr in bs.attributes() {
                let attr = attr?;
                if attr.key.starts_with(WEB_PREFIX) {
                    let value = String::from_utf8(attr.value.to_vec())?;
                    match &attr.key[WEB_PREFIX.len()..] {
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
                } else {
                    result.push_attribute(attr);
                }
            }
            Ok(result)
        };

        let mut buf = Vec::new();

        let mut reading_script = false;
        let mut current_script_type = None;
        let mut current_script = Vec::new();

        loop {
            const SCRIPT_TAG: &[u8] = b"web:script";

            match reader.read_event(&mut buf)? {
                Event::Eof => break,
                Event::Start(ref bs) => {
                    if bs.name() == SCRIPT_TAG{
                        match attrs_map(bs)?.get("type") {
                            Some(r#type) => match crate::script::Lang::from(r#type) {
                                Some(lang) => {
                                    current_script_type = Some(lang);
                                    current_script = Vec::new();
                                    reading_script = true;
                                },
                                None => println!("script tag found with unknown type '{}'", r#type),
                            },
                            None => println!("script tag found, but no type was specified"),
                        }
                    } else {
                        writer.write_event(Event::Start(trim_bytes_start(bs)?))?;
                    }
                },
                Event::Text(bt) => {
                    if reading_script {
                        current_script.append(&mut bt.unescaped()?.to_vec());
                    } else {
                        writer.write_event(Event::Text(bt))?;
                    }
                },
                Event::End(be) => {
                    if be.name() == SCRIPT_TAG {
                        if reading_script {
                            scripts.push((current_script_type.unwrap(), String::from_utf8(current_script.clone())?));
                            reading_script = false;
                        }
                    } else {
                        writer.write_event(Event::End(be))?;
                    }
                },
                Event::Empty(ref bs) => writer.write_event(Event::Empty(trim_bytes_start(bs)?))?,
                e => writer.write_event(&e)?,
            }
        }

        for (lang, script) in &scripts {
            println!("Found {:?} script: {}", &lang, &script);
        }

        Ok(Definition{
            buildable: String::from_utf8(writer.into_inner().into_inner())?,
            hrefs,
            scripts,
        })
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
}
