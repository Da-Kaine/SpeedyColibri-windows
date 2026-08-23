//! `coli chat <snap> [system_prompt...]` — terminal interactive chat REPL.
//!
//! Provides a live token-by-token streaming chat interface in the terminal for
//! supported models (GLM, Nemotron, MiniMax, Maple, etc.). Handles prompt template
//! formatting, history tracking, truncation on long context, and interactive REPL commands.

use std::io::{BufRead, Write};
use std::process::ExitCode;
use std::sync::Arc;

use colibri_engine::{
    generate_stream, load_model_with, ExpertCache, KvCache, LoadOptions, Model,
    ShardsExpertProvider, UsageHistory,
};
use colibri_json::{Json, JsonObj};
use colibri_tokenizer::Tokenizer;

use crate::serve::build_chat_prompt;

/// Default served context length (prompt + completion) when `COLI_CTX` is unset.
const DEFAULT_CTX: usize = 32_768;
/// Default max generated tokens per turn when `COLI_NGEN` is unset.
const DEFAULT_NGEN: usize = 512;

/// Helper to create a message Json object `{"role": role, "content": content}`.
pub(crate) fn create_message(role: &str, content: &str) -> Json {
    let mut obj = JsonObj::new();
    obj.push("role".to_string(), Json::Str(role.to_string()));
    obj.push("content".to_string(), Json::Str(content.to_string()));
    Json::Obj(obj)
}

/// Helper to parse integer environment variables.
fn envbits(k: &str, d: u32) -> u32 {
    std::env::var(k)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(d)
}

/// Derive model ID string for display.
fn model_id_from(snap: &str) -> String {
    let trimmed = snap.trim_end_matches('/');
    if let Some(pos) = trimmed.find("models--") {
        let seg = &trimmed[pos + "models--".len()..];
        let name = seg.split('/').next().unwrap_or(seg).replace("--", "/");
        if !name.is_empty() {
            return name;
        }
    }
    trimmed.rsplit('/').next().unwrap_or("model").to_string()
}

/// `coli chat <snapshot-dir> [system_prompt...]`
pub fn cmd_chat(args: &[String]) -> ExitCode {
    let snap = match args.get(2) {
        Some(p) => p.clone(),
        None => {
            eprintln!("usage: coli chat <snapshot-dir> [system_prompt...]");
            return ExitCode::from(2);
        }
    };
    crate::note_model_switch(&snap);

    let system_prompt = if args.len() > 3 {
        args[3..].join(" ")
    } else {
        std::env::var("COLI_SYSTEM").unwrap_or_default()
    };

    let opts = LoadOptions {
        dbits: envbits("COLI_DBITS", 8),
        ebits: envbits("COLI_EBITS", 8),
    };

    let model = match load_model_with(&snap, opts) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("coli chat: load model: {e}");
            return ExitCode::FAILURE;
        }
    };
    let model: &'static Model = Box::leak(Box::new(model));

    let tok_path = format!("{snap}/tokenizer.json");
    let tok = match Tokenizer::load(&tok_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("coli chat: load tokenizer ({tok_path}): {e}");
            return ExitCode::FAILURE;
        }
    };

    let model_id = model_id_from(&snap);

    // Setup cluster sharding & expert provider
    let usage_path = std::env::var("COLI_USAGE").unwrap_or_else(|_| format!("{snap}/.coli_usage"));
    let mut history = UsageHistory::load(&usage_path).unwrap_or_default();

    let cluster = colibri_cluster::ClusterConfig::from_env();
    let sharding = if cluster.is_single_node() {
        colibri_cluster::ExpertSharding::single(model.cfg.n_experts as u32)
    } else {
        crate::build_sharding(&cluster, model.cfg.n_experts as u32, &history)
    };

    let base = ShardsExpertProvider::with_sharding(
        &model.shards,
        &model.cfg,
        model.ebits as u32,
        sharding.clone(),
        cluster.this_node,
    );
    let budget = crate::ram_budget();
    let provider = Arc::new(ExpertCache::new(base, budget));
    let owned_ids: Vec<u32> = sharding.local_experts(cluster.this_node).collect();
    let maxres = crate::wire_adaptive_cache(
        &provider,
        &model.cfg,
        model.ebits as u32,
        &owned_ids,
        model.resident_bytes(),
    );
    crate::preload_all_experts(&provider, &model.cfg, maxres, &owned_ids);
    if let Some(topn) = crate::prefetch_topn() {
        provider.enable_prefetch(topn, model.cfg.n_experts as u64);
    }

    let own_history = crate::owned_history(&history, &sharding, cluster.this_node);
    crate::apply_autopin(&provider, &own_history, budget);

    crate::install_shutdown_handlers();

    let ctx_len = std::env::var("COLI_CTX")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DEFAULT_CTX)
        .min(if model.cfg.max_ctx > 0 {
            model.cfg.max_ctx as usize
        } else {
            usize::MAX
        });

    let default_ngen = envbits("COLI_NGEN", DEFAULT_NGEN as u32) as usize;

    let mut messages: Vec<Json> = Vec::new();
    if !system_prompt.trim().is_empty() {
        messages.push(create_message("system", system_prompt.trim()));
    }

    println!("colibrì terminal chat (model: {model_id})");
    println!("Type /exit or /quit to exit, /clear to reset conversation history, /help for help.");
    println!();

    let stdin = std::io::stdin();
    let mut handle_stdin = stdin.lock();

    while !crate::shutdown_requested() {
        print!("User: ");
        if std::io::stdout().flush().is_err() {
            break;
        }

        let mut line = String::new();
        match handle_stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("\nerror reading stdin: {e}");
                break;
            }
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("/exit") || input.eq_ignore_ascii_case("/quit") {
            println!("Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("/clear") || input.eq_ignore_ascii_case("/reset") {
            messages.clear();
            if !system_prompt.trim().is_empty() {
                messages.push(create_message("system", system_prompt.trim()));
            }
            println!("[Conversation history cleared]\n");
            continue;
        }

        if input.eq_ignore_ascii_case("/help") {
            println!("Commands:");
            println!("  /clear, /reset  Clear conversation history");
            println!("  /exit, /quit    Exit terminal chat");
            println!("  /help           Show this help message");
            println!();
            continue;
        }

        messages.push(create_message("user", input));

        // Build chat prompt token IDs
        let mut prompt_ids = build_chat_prompt(&tok, &messages, model.cfg.arch);

        // Truncate older history turns if prompt_ids exceeds context length limit
        while prompt_ids.len() >= ctx_len && messages.len() > 1 {
            let has_system = messages
                .first()
                .and_then(|m| m.get("role"))
                .and_then(|r| r.as_str())
                == Some("system");
            let remove_idx = if has_system && messages.len() > 2 { 1 } else { 0 };
            if remove_idx < messages.len() {
                messages.remove(remove_idx);
                prompt_ids = build_chat_prompt(&tok, &messages, model.cfg.arch);
            } else {
                break;
            }
        }

        if prompt_ids.len() >= ctx_len {
            eprintln!(
                "Prompt length ({} tokens) exceeds max context ({ctx_len} tokens). Please clear history or shorten prompt.",
                prompt_ids.len()
            );
            messages.pop(); // Remove the user input we just added
            continue;
        }

        let max_tokens = default_ngen.min(ctx_len.saturating_sub(prompt_ids.len()));
        if max_tokens == 0 {
            eprintln!("Context limit reached. Please type /clear to reset history.");
            messages.pop();
            continue;
        }

        let mut kv = KvCache::for_model(model, prompt_ids.len() + max_tokens);
        crate::charge_gen_kv(model, prompt_ids.len(), max_tokens);

        print!("Assistant: ");
        let _ = std::io::stdout().flush();

        let mut out_ids: Vec<i32> = Vec::with_capacity(max_tokens);
        let mut sent = String::new();

        let res = generate_stream(model, &mut kv, &*provider, &prompt_ids, max_tokens, |t| {
            if crate::shutdown_requested() {
                return false;
            }
            if model.cfg.stop_ids.contains(&t) {
                return false;
            }
            out_ids.push(t);
            let full = tok.decode(&out_ids);
            if full.len() > sent.len() {
                let delta = &full[sent.len()..];
                print!("{delta}");
                let _ = std::io::stdout().flush();
                sent = full;
            }
            true
        });

        println!("\n");

        if let Err(e) = res {
            eprintln!("[error during generation: {e}]");
        } else {
            messages.push(create_message("assistant", &sent));
        }

        // Merge usage history
        history.merge(&provider.usage_snapshot());
        let _ = history.save(&usage_path);
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_message() {
        let msg = create_message("user", "Hello world");
        assert_eq!(msg.get("role").and_then(|v| v.as_str()), Some("user"));
        assert_eq!(msg.get("content").and_then(|v| v.as_str()), Some("Hello world"));
    }

    #[test]
    fn test_model_id_from() {
        assert_eq!(
            model_id_from("/root/.cache/huggingface/hub/models--nvidia--GLM-5.2-NVFP4/snapshots/abc123"),
            "nvidia/GLM-5.2-NVFP4"
        );
        assert_eq!(model_id_from("/models/maple-preview/"), "maple-preview");
    }

    #[test]
    fn test_chat_history_truncation_logic() {
        let mut messages = vec![
            create_message("system", "You are helpful."),
            create_message("user", "Turn 1"),
            create_message("assistant", "Reply 1"),
            create_message("user", "Turn 2"),
        ];

        // Simulate truncation loop removing older non-system messages
        while messages.len() > 2 {
            let has_system = messages
                .first()
                .and_then(|m| m.get("role"))
                .and_then(|r| r.as_str())
                == Some("system");
            let remove_idx = if has_system && messages.len() > 2 { 1 } else { 0 };
            messages.remove(remove_idx);
        }

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].get("role").and_then(|v| v.as_str()), Some("system"));
        assert_eq!(messages[1].get("role").and_then(|v| v.as_str()), Some("user"));
        assert_eq!(messages[1].get("content").and_then(|v| v.as_str()), Some("Turn 2"));
    }
}
