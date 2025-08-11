use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;

use std::process::Command;

#[test]
fn test_organize_single_file() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Preparación
    let temp = TempDir::new()?;
    let config_content = r#"
[rules.Documentos]
extensions = ["pdf", "txt"]
"#;
    temp.child("config.toml").write_str(config_content)?;
    temp.child("reporte.pdf").write_str("Contenido del PDF de prueba")?;

    // 2. Ejecución
    let mut cmd = Command::cargo_bin("organizador_cli")?;
    cmd.current_dir(temp.path()); // Importante: ejecutar desde el directorio temporal
    cmd.arg("--directorio")
        .arg(".");
    
    cmd.assert().success(); // Comprueba que el programa termina sin errores

    // 3. Verificación
    temp.child("reporte.pdf").assert(predicates::path::missing());
    temp.child("Documentos/reporte.pdf").assert(predicates::path::is_file());

    Ok(())
}

#[test]
fn test_deduplicate_files() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Preparación
    let temp = TempDir::new()?;
    let config_content = r#"
[rules.Documentos]
extensions = ["txt"]
"#;
    temp.child("config.toml").write_str(config_content)?;
    temp.child("original.txt").write_str("mismo contenido")?;
    temp.child("copia.txt").write_str("mismo contenido")?; // Duplicado exacto
    temp.child("otro.txt").write_str("contenido diferente")?;

    // 2. Ejecución
    let mut cmd = Command::cargo_bin("organizador_cli")?;
    cmd.current_dir(temp.path());
    cmd.arg("--directorio")
        .arg(".");
    cmd.arg("--deduplicate"); // <-- Activamos la funcionalidad
    
    cmd.assert().success();

    // 3. Verificación
    // 'copia.txt' se procesa primero (orden alfabético), se conserva y se mueve.
    temp.child("copia.txt").assert(predicates::path::missing());
    temp.child("Documentos/copia.txt").assert(predicates::path::is_file());

    // 'original.txt' se procesa después, se detecta como duplicado y se elimina.
    temp.child("original.txt").assert(predicates::path::missing());
    temp.child("Documentos/original.txt").assert(predicates::path::missing());

    // El otro archivo no se ve afectado.
    temp.child("Documentos/otro.txt").assert(predicates::path::is_file());

    Ok(())
}
