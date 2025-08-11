use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
struct Rule {
    extensions: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Config {
    rules: HashMap<String, Rule>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, required = true)]
    directorio: PathBuf,
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
    #[arg(long)]
    dry_run: bool,
    /// Activa la detección y eliminación de archivos duplicados por contenido.
    #[arg(long)]
    deduplicate: bool,
}

/// Calcula el hash SHA-256 de un archivo y lo devuelve como un string hexadecimal.
fn file_hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let content = fs::read_to_string(&cli.config)
        .with_context(|| format!("No se pudo leer el archivo de configuración en {:?}", &cli.config))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| "No se pudo parsear el contenido del archivo de configuración.")?;

    println!("Directorio a organizar: {:?}", cli.directorio);
    if cli.dry_run {
        println!("Modo de simulación (dry-run) activado.");
    }
    if cli.deduplicate {
        println!("Detección de duplicados activada.");
    }

    // Registro para guardar hashes de archivos ya vistos
    let mut seen_hashes: HashMap<String, PathBuf> = HashMap::new();

    let mut entries: Vec<_> = fs::read_dir(&cli.directorio)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();

        if path.is_dir() || path.file_name() == Some(cli.config.as_os_str()) {
            continue;
        }

        // --- Lógica de Deduplicación ---
        if cli.deduplicate {
            let hash = file_hash(&path)?;
            if let Some(original_path) = seen_hashes.get(&hash) {
                println!("DUPLICADO: {:?} es un duplicado de {:?}", &path, original_path);
                if !cli.dry_run {
                    fs::remove_file(&path)?;
                    println!("--> Eliminado {:?}", &path);
                }
                continue; // No organizar este archivo, pasar al siguiente
            }
            seen_hashes.insert(hash, path.clone());
        }

        // --- Lógica de Organización ---
        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
            for (folder_name, rule) in &config.rules {
                if rule.extensions.iter().any(|ext| ext == extension) {
                    let dest_folder = cli.directorio.join(folder_name);

                    if cli.dry_run {
                        println!("[Dry Run] Movería {:?} a {:?}", &path, &dest_folder);
                    } else {
                        if !dest_folder.exists() {
                            fs::create_dir_all(&dest_folder)?;
                        }
                        let new_path = dest_folder.join(path.file_name().unwrap());
                        fs::rename(&path, &new_path)?;
                        println!("Movido {:?} a {:?}", &path, &new_path);
                    }
                    break;
                }
            }
        }
    }

    println!("\n¡Organización completada!");
    Ok(())
}