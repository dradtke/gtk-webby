pub fn absolutize_url(current_location: &String, target: &String) -> String {
    if target.is_empty() {
        return current_location.clone();
    }
    if target.contains("://") {
        return target.clone();
    }
    if !target.starts_with("/") {
        let mut result = String::new();
        result.push_str(current_location);
        result.push_str("/");
        result.push_str(target);
        return result;
    }
    match current_location.find("://") {
        Some(idx) => {
            let mut result = String::new();
            if let Some(root) = current_location[idx+3..].find("/") {
                result.push_str(&current_location[0..root+idx+3]);
            } else {
                    result.push_str(&current_location);
                }
            result.push_str(target);
            result
        },
        None => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_absolutize_url() {
        assert_eq!(absolutize_url(&String::new(), &String::from("http://localhost:8000")), "http://localhost:8000");
        assert_eq!(absolutize_url(&String::from("http://localhost:8000"), &String::from("/sub-page")), "http://localhost:8000/sub-page");
        assert_eq!(absolutize_url(&String::from("http://localhost:8000/sub-page"), &String::from("/another-page")), "http://localhost:8000/another-page");
        assert_eq!(absolutize_url(&String::from("http://localhost:8000/sub-page"), &String::from("sub-sub-page")), "http://localhost:8000/sub-page/sub-sub-page");
        assert_eq!(absolutize_url(&String::from("http://localhost:8000/current-page"), &String::from("")), "http://localhost:8000/current-page");
    }
}
