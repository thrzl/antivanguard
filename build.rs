extern crate embed_resource;
extern crate winres;


fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("manifest.xml");
    res.compile().unwrap();

    embed_resource::compile("tray.rc", embed_resource::NONE);
}