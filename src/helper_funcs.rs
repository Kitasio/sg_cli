use url::Url;

pub fn increment_image_stage(path: &str, new_stage: u8) -> String {
    let mut image_url = Url::parse(path).expect("Failed to parse image url");
    let mut segments = image_url
        .path_segments()
        .ok_or_else(|| "cannot be base")
        .expect("Failed to split to segments");

    let folder = segments
        .next()
        .expect("Could not resolve main folder from Url");
    segments.next().expect("Could not resolve stage from Url");
    let file = segments
        .next()
        .expect("Could not resolve the image file from Url");

    let new_path = format!("{}/{}/{}", folder, new_stage, file);

    image_url.set_path(new_path.as_str());
    image_url.to_string()
}