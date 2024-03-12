use crate::replace;
use crate::replace::Replace;
use clap::Parser;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct Generate {
    name: String,

    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    #[arg(short, long)]
    cover: bool,

    #[arg(short, long, default_value = "template/template")]
    template: PathBuf,
}

impl Generate {
    pub fn run(self) -> anyhow::Result<()> {
        if !self.template.exists() {
            eprintln!("Error: 模板不存在");
            return Ok(());
        }

        let reps = replace!("name" => &self.name);
        let out_path = self.path.join(&self.name);
        let mod_path = self.path.join("mod.rs");
        let mod_data = fs::read_to_string(&mod_path).unwrap_or_default();
        fs::write(mod_path, format!("mod {};\n{mod_data}", self.name))?;

        if out_path.exists() && !self.cover {
            let out_path = out_path.to_string_lossy().replace('\\', "/");
            eprintln!("目录 {out_path} 已经存在, -c 覆盖",);
            return Ok(());
        }

        fs::create_dir_all(&out_path)?;
        if let Err(err) = generate(self.template, out_path, &reps) {
            eprintln!("Error: {err}")
        }
        Ok(())
    }
}

fn generate(input: PathBuf, output: PathBuf, reps: &Replace) -> anyhow::Result<()> {
    fs::read_dir(&input)?
        .par_bridge()
        .flatten()
        .for_each(|entry| {
            let meta = entry.metadata().unwrap();
            let name = entry.file_name();
            let path = entry.path();
            let out_name = reps.replace(name.to_string_lossy());

            if meta.is_file() {
                if let Err(err) = fs::read_to_string(&path).and_then(|mut data| {
                    data = reps.replace(data);
                    fs::write(output.join(&out_name), data)
                }) {
                    eprintln!("Error: {err}")
                }
                return;
            }
            if meta.is_dir() {
                if let Err(err) = generate(input.join(&name), output.join(&out_name), reps) {
                    eprintln!("Error: {err}")
                };
            }
        });
    Ok(())
}
