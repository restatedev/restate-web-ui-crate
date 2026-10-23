// Copyright (c) 2023 - 2025 Restate Software, Inc., Restate GmbH.
// All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

// The UI release is an uncompressed ustar archive, so the files are served
// straight out of it. build.rs locates each file in it.
static ARCHIVE: &[u8] = include_bytes!(env!("UI_ARCHIVE"));

pub static ASSETS: Assets = Assets;

pub struct Assets;

impl Assets {
    pub fn get_file(&self, path: &str) -> Option<&'static File> {
        include!(concat!(env!("OUT_DIR"), "/lookup.rs"))
    }
}

pub struct File {
    start: usize,
    end: usize,
}

impl File {
    pub fn contents(&self) -> &'static [u8] {
        &ARCHIVE[self.start..self.end]
    }
}

#[cfg(test)]
mod tests {
    use super::ASSETS;

    #[test]
    fn serves_files_from_the_archive() {
        let index = ASSETS
            .get_file("index.html")
            .expect("index.html must be present");
        assert!(index.contents().starts_with(b"<!DOCTYPE html>"));
        assert!(ASSETS.get_file("missing.js").is_none());
    }
}
