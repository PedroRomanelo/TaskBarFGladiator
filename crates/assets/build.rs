//! Empacota as folhas de sprites de `assets-src/sprites/<conjunto>/` em um formato binário compacto,
//! embutido no executável com `include_bytes!`. O decodificador de PNG só existe aqui, no build: o
//! executável lê o formato próprio (D25).
//!
//! Cada conjunto tem um `set.json` (ver `assets-src/sprites/knight/set.json`):
//!
//! - `frame_w`, `frame_h`: tamanho de cada quadro na folha (quadros lado a lado, da esquerda para a
//!   direita);
//! - `anchor_x`, `floor_y`: coluna do eixo do corpo e primeira linha abaixo dos pés, dentro do quadro;
//! - `anims`: lista de `{ anim, file, ms, loop?, frames? }`; `anim` é um dos nomes de
//!   `ANIM_NAMES`; `frames` escolhe quadros da folha (padrão: todos);
//! - `roles`: cores (`#rrggbb`) que o jogo troca: `team` (3 tons, do claro ao escuro), `skin`
//!   (3 tons), `eyes` (1 tom) e `armor` (3 tons).
//!
//! Formato de saída (little-endian), lido por `crates/assets/src/sprites.rs`:
//!
//! ```text
//! "GSP1"
//! u8  n               cores na paleta, contando o índice 0 (transparente)
//! n-1 × (r, g, b)     cores 1..n
//! n-1 × u8            papel de cada cor (0 fixa; 0x10+k time; 0x20+k pele; 0x30 olhos; 0x40+k armadura)
//! u8  animações
//!   u8 id, u16 ms, u8 loop, u8 quadros
//!     i16 x, i16 y    canto superior esquerdo relativo ao eixo e ao chão
//!     u16 w, u16 h
//!     u32 bytes       seguidos dos pares (repetições 1..=255, índice)
//! ```

use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Nomes das animações, na ordem dos identificadores de `assets::sprites::Anim`.
const ANIM_NAMES: [&str; 9] = [
    "idle",
    "walk",
    "attack_light",
    "attack_medium",
    "attack_heavy",
    "guard",
    "block",
    "hurt",
    "death",
];

const MAX_COLORS: usize = 64;

type BuildResult<T> = Result<T, String>;

fn main() {
    if let Err(e) = run() {
        // Erro de arte: falha o build com a mensagem, em vez de gerar um conjunto quebrado.
        println!("cargo::error={e}");
    }
}

fn run() -> BuildResult<()> {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?);
    let out = PathBuf::from(std::env::var("OUT_DIR").map_err(|e| e.to_string())?);
    let root = manifest.join("../../assets-src/sprites");
    println!("cargo::rerun-if-changed={}", root.display());

    let mut sets = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&root) {
        let mut names: Vec<PathBuf> = dir
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.join("set.json").is_file())
            .collect();
        names.sort();
        for path in names {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or("nome de pasta inválido")?
                .to_owned();
            let bytes = pack(&path).map_err(|e| format!("sprites/{name}: {e}"))?;
            let file = out.join(format!("sprites_{name}.bin"));
            std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
            sets.push((name, file));
        }
    }

    let mut code =
        String::from("/// Conjuntos de sprites encontrados em `assets-src/sprites/` no build.\n");
    code.push_str("pub static SETS: &[(&str, &[u8])] = &[\n");
    for (name, file) in &sets {
        let _ = writeln!(
            code,
            "    ({name:?}, include_bytes!({:?})),",
            file.display().to_string()
        );
    }
    code.push_str("];\n");
    std::fs::write(out.join("sprite_sets.rs"), code).map_err(|e| e.to_string())
}

struct Sheet {
    w: usize,
    h: usize,
    rgba: Vec<u8>,
}

fn load_png(path: &Path) -> BuildResult<Sheet> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| format!("{}: imagem grande demais", path.display()))?;
    let mut buf = vec![0; size];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let (w, h) = (info.width as usize, info.height as usize);
    let mut rgba = Vec::with_capacity(w * h * 4);
    for px in buf[..info.buffer_size()].chunks(info.color_type.samples()) {
        match px {
            [r, g, b, a] => rgba.extend_from_slice(&[*r, *g, *b, *a]),
            [r, g, b] => rgba.extend_from_slice(&[*r, *g, *b, 255]),
            [l, a] => rgba.extend_from_slice(&[*l, *l, *l, *a]),
            [l] => rgba.extend_from_slice(&[*l, *l, *l, 255]),
            _ => return Err(format!("{}: formato de cor inesperado", path.display())),
        }
    }
    Ok(Sheet { w, h, rgba })
}

fn hex(v: &Value) -> BuildResult<u32> {
    let s = v.as_str().ok_or("cor deve ser texto \"#rrggbb\"")?;
    u32::from_str_radix(s.trim_start_matches('#'), 16).map_err(|e| format!("cor {s}: {e}"))
}

fn num(v: &Value, key: &str) -> BuildResult<i64> {
    v.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("campo numérico \"{key}\" ausente"))
}

fn pack(dir: &Path) -> BuildResult<Vec<u8>> {
    let text = std::fs::read_to_string(dir.join("set.json")).map_err(|e| e.to_string())?;
    let set: Value = serde_json::from_str(&text).map_err(|e| format!("set.json: {e}"))?;
    let fw = num(&set, "frame_w")? as usize;
    let fh = num(&set, "frame_h")? as usize;
    let ax = num(&set, "anchor_x")? as i32;
    let floor = num(&set, "floor_y")? as i32;

    // Papéis das cores.
    let mut roles: HashMap<u32, u8> = HashMap::new();
    if let Some(r) = set.get("roles") {
        for (key, base, max) in [
            ("team", 0x10, 3),
            ("skin", 0x20, 3),
            ("eyes", 0x30, 1),
            ("armor", 0x40, 3),
        ] {
            if let Some(list) = r.get(key).and_then(Value::as_array) {
                if list.len() > max {
                    return Err(format!("roles.{key}: no máximo {max} cores"));
                }
                for (k, c) in list.iter().enumerate() {
                    roles.insert(hex(c)?, base + k as u8);
                }
            }
        }
    }

    let mut palette: Vec<u32> = Vec::new();
    let mut index_of = |rgb: u32| -> BuildResult<u8> {
        if let Some(i) = palette.iter().position(|&c| c == rgb) {
            return Ok(i as u8 + 1);
        }
        if palette.len() + 1 >= MAX_COLORS {
            return Err(format!("mais de {} cores", MAX_COLORS - 1));
        }
        palette.push(rgb);
        Ok(palette.len() as u8)
    };

    let mut anims = Vec::new();
    let mut seen = [false; ANIM_NAMES.len()];
    for a in set
        .get("anims")
        .and_then(Value::as_array)
        .ok_or("lista \"anims\" ausente")?
    {
        let name = a
            .get("anim")
            .and_then(Value::as_str)
            .ok_or("anim sem nome")?;
        let id = ANIM_NAMES
            .iter()
            .position(|n| *n == name)
            .ok_or_else(|| format!("animação desconhecida \"{name}\" (use {ANIM_NAMES:?})"))?;
        if std::mem::replace(&mut seen[id], true) {
            return Err(format!("animação \"{name}\" repetida"));
        }
        let file = a
            .get("file")
            .and_then(Value::as_str)
            .ok_or("anim sem \"file\"")?;
        let sheet = load_png(&dir.join(file))?;
        if sheet.w % fw != 0 || sheet.h != fh {
            return Err(format!(
                "{file}: {}x{} não é uma fila de quadros {fw}x{fh}",
                sheet.w, sheet.h
            ));
        }
        let count = sheet.w / fw;
        let picks: Vec<usize> = match a.get("frames").and_then(Value::as_array) {
            Some(list) => list
                .iter()
                .map(|v| {
                    v.as_u64()
                        .map(|n| n as usize)
                        .filter(|&n| n < count)
                        .ok_or_else(|| format!("{file}: quadro inválido {v}"))
                })
                .collect::<BuildResult<_>>()?,
            None => (0..count).collect(),
        };
        let ms = num(a, "ms")?.clamp(1, 60_000) as u16;
        let looped = a.get("loop").and_then(Value::as_bool).unwrap_or(false);

        let mut frames = Vec::new();
        for f in picks {
            // Caixa dos pixels visíveis do quadro.
            let (mut x0, mut y0, mut x1, mut y1) = (fw, fh, 0, 0);
            for y in 0..fh {
                for x in 0..fw {
                    if sheet.rgba[(y * sheet.w + f * fw + x) * 4 + 3] >= 128 {
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x + 1);
                        y1 = y1.max(y + 1);
                    }
                }
            }
            if x1 == 0 {
                (x0, y0, x1, y1) = (0, 0, 0, 0);
            }
            let mut idx = Vec::with_capacity((x1 - x0) * (y1 - y0));
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = (y * sheet.w + f * fw + x) * 4;
                    let p = &sheet.rgba[i..i + 4];
                    // Pixel art: alfa é tudo ou nada.
                    if p[3] < 128 {
                        idx.push(0);
                    } else {
                        let rgb =
                            (u32::from(p[0]) << 16) | (u32::from(p[1]) << 8) | u32::from(p[2]);
                        idx.push(index_of(rgb)?);
                    }
                }
            }
            frames.push((
                x0 as i32 - ax,
                y0 as i32 - floor,
                x1 - x0,
                y1 - y0,
                rle(&idx),
            ));
        }
        anims.push((id as u8, ms, looped, frames));
    }
    if !seen[0] {
        return Err("a animação \"idle\" é obrigatória".into());
    }

    let mut out = b"GSP1".to_vec();
    out.push(palette.len() as u8 + 1);
    for c in &palette {
        out.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, *c as u8]);
    }
    for c in &palette {
        out.push(roles.get(c).copied().unwrap_or(0));
    }
    out.push(anims.len() as u8);
    for (id, ms, looped, frames) in anims {
        out.push(id);
        out.extend_from_slice(&ms.to_le_bytes());
        out.push(u8::from(looped));
        out.push(frames.len() as u8);
        for (x, y, w, h, data) in frames {
            out.extend_from_slice(&(x as i16).to_le_bytes());
            out.extend_from_slice(&(y as i16).to_le_bytes());
            out.extend_from_slice(&(w as u16).to_le_bytes());
            out.extend_from_slice(&(h as u16).to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&data);
        }
    }
    Ok(out)
}

fn rle(idx: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < idx.len() {
        let v = idx[i];
        let mut n = 1;
        while i + n < idx.len() && idx[i + n] == v && n < 255 {
            n += 1;
        }
        out.push(n as u8);
        out.push(v);
        i += n;
    }
    out
}
