//! `embed` — pure-Rust port of `system1_embed.py` (sentence-transformers
//! all-MiniLM-L6-v2: BERT encoder + mean pooling + L2 normalize, cosine
//! top-k). Reads `{"task","candidates":{id:criterion},"k"}` on stdin and
//! prints `{"names":[...]}`; any error exits non-zero with stdout empty so
//! callers fall back to the lexical scorer.
//!
//! Model-dir contract: `CIEL_EMBED_MODEL_DIR` overrides; otherwise the
//! Hugging Face cache snapshot for `sentence-transformers/all-MiniLM-L6-v2`
//! under `~/.cache/huggingface/hub` (or `$HF_HUB_CACHE`/`$HF_HOME`). The
//! directory must contain `model.safetensors`, `tokenizer.json`, and
//! `config.json`. Nothing is ever downloaded.

#[cfg(feature = "embed")]
mod imp {
    use candle_core::{Device, Tensor};
    use candle_nn::VarBuilder;
    use candle_transformers::models::bert::{BertModel, Config, DTYPE};
    use serde_json::Value;
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use tokenizers::{PaddingParams, Tokenizer, TruncationParams};

    /// `~/.ciel/system1/venv`'s HF cache layout is the stock
    /// `~/.cache/huggingface/hub` — resolve the snapshot for
    /// `sentence-transformers/all-MiniLM-L6-v2` via `refs/main`, else the
    /// newest `snapshots/*` dir carrying the required files.
    pub fn model_dir() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var("CIEL_EMBED_MODEL_DIR") {
            if !dir.is_empty() {
                return Some(PathBuf::from(dir));
            }
        }
        let hub = std::env::var("HF_HUB_CACHE")
            .map(PathBuf::from)
            .ok()
            .or_else(|| {
                std::env::var("HF_HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join("hub"))
            })
            .or_else(|| {
                Some(
                    crate::py::home_dir()
                        .join(".cache")
                        .join("huggingface")
                        .join("hub"),
                )
            })?;
        let model_root = hub.join("models--sentence-transformers--all-MiniLM-L6-v2");
        let snapshots = model_root.join("snapshots");
        if let Ok(refs) = std::fs::read_to_string(model_root.join("refs").join("main")) {
            let cand = snapshots.join(refs.trim());
            if cand.join("model.safetensors").is_file() && cand.join("tokenizer.json").is_file() {
                return Some(cand);
            }
        }
        let mut best: Option<PathBuf> = None;
        if let Ok(rd) = std::fs::read_dir(&snapshots) {
            for e in rd.flatten() {
                let p = e.path();
                if p.join("model.safetensors").is_file() && p.join("tokenizer.json").is_file() {
                    best = Some(p);
                }
            }
        }
        best
    }

    struct Encoder {
        tokenizer: Tokenizer,
        model: BertModel,
        device: Device,
    }

    fn encoder() -> Option<&'static Encoder> {
        static ENC: OnceLock<Option<Encoder>> = OnceLock::new();
        ENC.get_or_init(|| {
            let dir = model_dir()?;
            let config: Config =
                serde_json::from_str(&std::fs::read_to_string(dir.join("config.json")).ok()?)
                    .ok()?;
            let device = Device::Cpu;
            let vb = unsafe {
                VarBuilder::from_mmaped_safetensors(
                    &[dir.join("model.safetensors")],
                    DTYPE,
                    &device,
                )
                .ok()?
            };
            let model = BertModel::load(vb, &config).ok()?;
            let mut tokenizer = Tokenizer::from_file(dir.join("tokenizer.json")).ok()?;
            // sentence_bert_config.json: max_seq_length=256.
            let _ = tokenizer.with_truncation(Some(TruncationParams {
                max_length: 256,
                ..Default::default()
            }));
            // Pad only when batching — we encode single sequences.
            let _ = tokenizer.with_padding(None::<PaddingParams>);
            Some(Encoder {
                tokenizer,
                model,
                device,
            })
        })
        .as_ref()
    }

    /// One document → L2-normalized sentence embedding (mean pooling over
    /// `last_hidden_state` weighted by the attention mask).
    fn encode(enc: &Encoder, text: &str) -> Option<Vec<f32>> {
        let encoding = enc.tokenizer.encode(text, true).ok()?;
        let ids: Vec<u32> = encoding.get_ids().to_vec();
        let type_ids: Vec<u32> = encoding.get_type_ids().to_vec();
        let mask: Vec<u32> = encoding.get_attention_mask().to_vec();
        let seq = ids.len();
        let ids = Tensor::from_vec(ids, (1, seq), &enc.device).ok()?;
        let type_ids = Tensor::from_vec(type_ids, (1, seq), &enc.device).ok()?;
        let mask_t = Tensor::from_vec(mask, (1, seq), &enc.device).ok()?;
        let hidden = enc.model.forward(&ids, &type_ids, Some(&mask_t)).ok()?; // [1, seq, hidden]
        let mask_f = mask_t
            .to_dtype(candle_core::DType::F32)
            .ok()?
            .unsqueeze(2)
            .ok()? // [1, seq, 1]
            .broadcast_as(hidden.shape())
            .ok()?;
        let summed = hidden.mul(&mask_f).ok()?.sum(1).ok()?; // [1, hidden]
        let denom = mask_f.sum(1).ok()?; // [1, hidden] broadcast
        let emb = summed.div(&denom).ok()?.squeeze(0).ok()?; // [hidden]
                                                             // L2 normalize (normalize_embeddings=True).
        let norm = emb.sqr().ok()?.sum_all().ok()?.sqrt().ok()?;
        let emb = emb.broadcast_div(&norm).ok()?;
        emb.to_vec1::<f32>().ok()
    }

    /// `top_k(task, candidates, k)` — names ranked by cosine similarity
    /// (numpy `sims.argsort()[::-1][:k]`).
    pub fn top_k(task: &str, candidates: &[(String, String)], k: usize) -> Option<Vec<String>> {
        let enc = encoder()?;
        let docs: Vec<Vec<f32>> = candidates
            .iter()
            .map(|(n, c)| encode(enc, &format!("{n}: {c}")))
            .collect::<Option<_>>()?;
        let q = encode(enc, task)?;
        let mut sims: Vec<(f32, usize)> = docs
            .iter()
            .enumerate()
            .map(|(i, d)| (d.iter().zip(&q).map(|(a, b)| a * b).sum(), i))
            .collect();
        // argsort ascending, then reversed — stable ties like numpy's
        // default quicksort are unobservable at f32 resolution.
        sims.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        sims.reverse();
        Some(
            sims.iter()
                .take(k)
                .map(|(_, i)| candidates[*i].0.clone())
                .collect(),
        )
    }

    /// `main_` — stdin JSON in, `{"names": [...]}` out, 0/1 exit.
    /// `--help` short-circuits before the stdin read so it stays side-effect
    /// free (the Python helper ignores argv entirely).
    pub fn main_(args: &[String]) -> i32 {
        for a in args {
            if a == "-h" || a == "--help" {
                println!(
                    "usage: ciel-dev embed\n  reads {{\"task\",\"candidates\":{{id:criterion}},\"k\"}} on stdin,\n  prints {{\"names\":[...]}}; model dir via CIEL_EMBED_MODEL_DIR"
                );
                return 0;
            }
        }
        let run = || -> Option<String> {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).ok()?;
            let payload: Value = serde_json::from_str(&buf).unwrap_or(json_obj());
            let task = payload.get("task").and_then(|v| v.as_str()).unwrap_or("");
            let candidates = payload
                .get("candidates")
                .and_then(|c| c.as_object())
                .cloned()
                .unwrap_or_default();
            let k = payload
                .get("k")
                .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()))
                .unwrap_or(10.0) as usize;
            if task.is_empty() || candidates.is_empty() {
                return None;
            }
            let pairs: Vec<(String, String)> = candidates
                .iter()
                .map(|(n, c)| {
                    (
                        n.clone(),
                        c.as_str()
                            .map(String::from)
                            .unwrap_or_else(|| crate::jsonfmt::dumps_raw(c)),
                    )
                })
                .collect();
            let names = top_k(task, &pairs, k)?;
            Some(crate::jsonfmt::dumps(&serde_json::json!({"names": names})))
        };
        match run() {
            Some(out) => {
                println!("{out}");
                0
            }
            None => 1,
        }
    }

    fn json_obj() -> Value {
        Value::Object(serde_json::Map::new())
    }
}

#[cfg(feature = "embed")]
pub use imp::{main_, top_k};

#[cfg(not(feature = "embed"))]
pub fn main_(args: &[String]) -> i32 {
    for a in args {
        if a == "-h" || a == "--help" {
            println!("usage: ciel-dev embed");
            return 0;
        }
    }
    // Same contract as the Python helper: fail non-zero, stdout empty.
    1
}
