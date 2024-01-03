use std::env;
use fs_extra::dir::CopyOptions;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=res/*");
    fs_extra::copy_items(&["res/"], env::var("OUT_DIR")?, &CopyOptions::new().overwrite(true))?;
    Ok(())
}
