// Copyright (c) 2023 - 2025 Restate Software, Inc., Restate GmbH.
// All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::env;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

const BLOCK: usize = 512;

// Indexes the UI archive (uncompressed ustar) and generates `lookup.rs`, a
// `match` from each file path to its byte range in the archive.
fn main() {
    let archive_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(format!(
        "assets/ui-v{}.tar",
        env::var("CARGO_PKG_VERSION").unwrap()
    ));
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rustc-env=UI_ARCHIVE={}", archive_path.display());

    let archive = fs::read(&archive_path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", archive_path.display()));

    let mut lookup = String::from("match path {\n");
    let mut has_index = false;
    let mut offset = 0;

    // The archive ends with zero-filled blocks.
    while let Some(header) = archive
        .get(offset..offset + BLOCK)
        .filter(|header| header[0] != 0)
    {
        assert_eq!(
            &header[257..263],
            b"ustar\0",
            "UI archive must be in ustar format"
        );
        let size = usize::from_str_radix(field(&header[124..136]).trim(), 8)
            .expect("tar entry size must be octal");
        let start = offset + BLOCK;
        let end = start + size;
        assert!(end <= archive.len(), "UI archive is truncated");

        match header[156] {
            b'0' | b'\0' => {
                let name = field(&header[0..100]);
                let path = match field(&header[345..500]) {
                    "" => name.to_owned(),
                    prefix => format!("{prefix}/{name}"),
                };
                let path = path.strip_prefix("./").unwrap_or(&path);
                has_index |= path == "index.html";
                writeln!(
                    lookup,
                    "    {path:?} => Some(&File {{ start: {start}, end: {end} }}),"
                )
                .unwrap();
            }
            b'5' => {}
            other => panic!("unsupported tar entry type {:?}", other as char),
        }

        offset = start + size.next_multiple_of(BLOCK);
    }

    assert!(has_index, "UI archive must contain index.html");
    lookup.push_str("    _ => None,\n}\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("lookup.rs"), lookup).unwrap();
}

fn field(bytes: &[u8]) -> &str {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..end]).expect("tar header fields must be UTF-8")
}
