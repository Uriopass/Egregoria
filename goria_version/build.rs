use vergen::Config;

fn main() {
    let mut c = Config::default();
    *c.git_mut().commit_timestamp_mut() = false;
    *c.git_mut().branch_mut() = false;
    *c.git_mut().sha_kind_mut() = vergen::ShaKind::Short;
    let _ = vergen::vergen(c);
}
