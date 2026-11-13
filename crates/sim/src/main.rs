//! Simulador headless de balanceamento.
//!
//! Uso:
//! ```text
//! sim metrics [--quick]                 imprime as métricas da seção 3 de docs/12
//! sim check [--quick] [--targets F]     falha (código 1) se alguma métrica sair da meta
//! sim write-baseline [--dir D]          grava D/baseline.csv e D/baseline-careers.csv
//! sim verify-baseline [--dir D]         falha se o resultado diferir do baseline gravado
//! sim compare [--dir D]                 grava D/compare.md (baseline × atual)
//! sim trace [--seed N] [--level L]      imprime uma luta turno a turno
//! sim career-trace [--seed N]           imprime uma carreira luta a luta
//! ```
//! Ver `docs/12-balanceamento-de-referencia.md`.
#![forbid(unsafe_code)]

mod career;
mod fights;
mod metrics;

use game::Content;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Tamanho da simulação.
#[derive(Debug, Clone, Copy)]
pub struct Scale {
    /// Lutas por cenário.
    pub fights: u32,
    /// Carreiras.
    pub careers: u32,
}

impl Scale {
    const FULL: Scale = Scale {
        fights: 10_000,
        careers: 1_000,
    };
    const QUICK: Scale = Scale {
        fights: 1_000,
        careers: 100,
    };
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("metrics");
    let scale = if args.iter().any(|a| a == "--quick") {
        Scale::QUICK
    } else {
        Scale::FULL
    };
    let dir = PathBuf::from(arg_value(&args, "--dir").unwrap_or_else(|| "balance".into()));
    let content = match Content::embedded() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("erro no conteúdo: {e}");
            return ExitCode::FAILURE;
        }
    };
    match command {
        "metrics" => {
            let m = metrics::compute(&content, scale);
            print!("{}", metrics::to_table(&m, None));
            ExitCode::SUCCESS
        }
        "check" => {
            let targets_path = arg_value(&args, "--targets")
                .map(PathBuf::from)
                .unwrap_or_else(|| dir.join("targets.json"));
            check(&content, scale, &targets_path)
        }
        "write-baseline" => match write_baseline(&content, &dir) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        "verify-baseline" => verify_baseline(&content, &dir),
        "compare" => compare(&content, &dir),
        "career-trace" => {
            let seed = arg_value(&args, "--seed")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            let mut log = String::new();
            let r = career::run_traced(&content, seed, Some(&mut log));
            println!("{log}{r:?}");
            ExitCode::SUCCESS
        }
        "trace" => {
            let seed = arg_value(&args, "--seed")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            let level = arg_value(&args, "--level")
                .and_then(|v| v.parse().ok())
                .unwrap_or(9);
            print!("{}", fights::trace(&content, seed, level));
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("comando desconhecido: {other}");
            ExitCode::FAILURE
        }
    }
}

/// Meta de uma métrica.
#[derive(Debug, Clone, serde::Deserialize)]
struct Target {
    min: Option<f64>,
    max: Option<f64>,
}

fn check(content: &Content, scale: Scale, path: &Path) -> ExitCode {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("não foi possível ler {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let targets: BTreeMap<String, Target> = match serde_json::from_str(&text) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let m = metrics::compute(content, scale);
    let mut failed = 0;
    println!("| Meta | Valor | Mín | Máx | OK |");
    println!("|---|---|---|---|---|");
    for (id, t) in &targets {
        let Some(&v) = m.get(id) else {
            println!("| {id} | — | | | métrica inexistente |");
            failed += 1;
            continue;
        };
        let ok = t.min.is_none_or(|lo| v >= lo) && t.max.is_none_or(|hi| v <= hi);
        if !ok {
            failed += 1;
        }
        let fmt = |x: Option<f64>| x.map(|x| format!("{x}")).unwrap_or_default();
        println!(
            "| {id} | {v:.3} | {} | {} | {} |",
            fmt(t.min),
            fmt(t.max),
            if ok { "sim" } else { "**NÃO**" }
        );
    }
    if failed > 0 {
        eprintln!("{failed} meta(s) fora da faixa");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn baseline_texts(content: &Content) -> (String, String) {
    let header = format!("# balance_version={}\n", content.balance_version);
    let fights = fights::run_matrix(content, Scale::FULL.fights);
    let careers = career::run_many(content, Scale::FULL.careers);
    (
        format!("{header}{}", fights::to_csv(&fights)),
        format!("{header}{}", career::to_csv(&careers)),
    )
}

fn write_baseline(content: &Content, dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let (f, c) = baseline_texts(content);
    std::fs::write(dir.join("baseline.csv"), f)?;
    std::fs::write(dir.join("baseline-careers.csv"), c)?;
    let m = metrics::compute(content, Scale::FULL);
    std::fs::write(dir.join("baseline-metrics.json"), metrics::to_json(&m))?;
    println!("baseline gravado em {}", dir.display());
    Ok(())
}

fn verify_baseline(content: &Content, dir: &Path) -> ExitCode {
    let (f, c) = baseline_texts(content);
    let mut ok = true;
    for (name, now) in [("baseline.csv", f), ("baseline-careers.csv", c)] {
        let path = dir.join(name);
        match std::fs::read_to_string(&path) {
            Ok(saved) if saved == now => println!("{name}: idêntico"),
            Ok(saved) => {
                ok = false;
                eprintln!("{name}: DIFERENTE do baseline (mudança de balanceamento não declarada)");
                for (i, (a, b)) in saved.lines().zip(now.lines()).enumerate() {
                    if a != b {
                        eprintln!("  linha {}:\n    baseline: {a}\n    atual:    {b}", i + 1);
                    }
                }
            }
            Err(e) => {
                ok = false;
                eprintln!("{}: {e}", path.display());
            }
        }
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn compare(content: &Content, dir: &Path) -> ExitCode {
    let path = dir.join("baseline-metrics.json");
    let saved: BTreeMap<String, f64> = match std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string()))
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let now = metrics::compute(content, Scale::FULL);
    let table = metrics::to_table(&now, Some(&saved));
    let out = dir.join("compare.md");
    if let Err(e) = std::fs::write(&out, &table) {
        eprintln!("{}: {e}", out.display());
        return ExitCode::FAILURE;
    }
    print!("{table}");
    ExitCode::SUCCESS
}
