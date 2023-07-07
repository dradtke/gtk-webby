use reqwest::header::{HeaderMap, HeaderName};

/// This function defines the headers that should be included on every request.
pub fn request_headers() -> HeaderMap {
    let headers = vec![
        (
            HeaderName::from_static("x-gtk-version-major"),
            gtk::major_version().to_string().parse().unwrap(),
        ),
        (
            HeaderName::from_static("x-gtk-version-minor"),
            gtk::minor_version().to_string().parse().unwrap(),
        ),
        (
            HeaderName::from_static("x-gtk-version-micro"),
            gtk::micro_version().to_string().parse().unwrap(),
        ),
    ];
    reqwest::header::HeaderMap::from_iter(headers)
}
