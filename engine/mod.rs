// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro2::TokenStream as TokenStream2;

use quote::ToTokens;
use syn::parse::{Parse, ParseStream, Result};

use syn::ItemMod;

use log::debug;

pub enum CppInclusion {
    Define(String),
    Header(String),
}

pub struct IncludeCpp<'a> {
    inclusions: Vec<CppInclusion>,
    allowlist: Vec<&'a str>,
    inc_dir: &'a str, // TODO make more versatile
}

impl<'a> Parse for IncludeCpp<'a> {
    fn parse(_input: ParseStream) -> Result<Self> {
        // TODO: Takes as inputs:
        // 1. List of headers to include
        // 2. List of #defines to include
        // 3. Allowlist
        Ok(IncludeCpp {
            inclusions: vec![],
            allowlist: vec![],
            inc_dir: "",
        })
    }
}

impl<'a> IncludeCpp<'a> {
    #[cfg(test)]
    pub fn new(inclusions: Vec<CppInclusion>,
        allowlist: Vec<&'a str>,
        inc_dir: &'a str) -> Self {
        IncludeCpp {
            inclusions,
            allowlist,
            inc_dir
        }
    }

    fn build_header(&self) -> String {
        let mut s = String::new();
        for incl in &self.inclusions {
            let text = match incl {
                CppInclusion::Define(symbol) => format!("#define {}\n", symbol),
                CppInclusion::Header(path) => format!("#include \"{}\"\n", path),
            };
            s.push_str(&text);
        }
        s
    }

    fn make_builder(&self) -> bindgen::Builder {
        let full_header = self.build_header();
        debug!("Full header: {}", full_header);

        // TODO - pass headers in &self.inclusions into
        // bindgen such that it can include them in the generated
        // extern "C" section as include!
        // The .hpp below is important so bindgen works in C++ mode
        let mut builder = bindgen::builder()
            .clang_arg(format!("-I{}", self.inc_dir))
            .header_contents("example.hpp", &full_header);
        // 3. Passes allowlist and other options to the bindgen::Builder equivalent
        //    to --output-style=cxx --allowlist=<as passed in>
        for a in &self.allowlist {
            // TODO - allowlist type/functions/separately
            builder = builder.whitelist_type(a);
            builder = builder.whitelist_function(a);
        }
        builder
    }

    pub fn run(self) -> TokenStream2 {
        // TODO:
        // 4. (also respects environment variables to pick up more headers,
        //     include paths and #defines)
        // Then:
        // 1. Builds an overall C++ header with all those #defines and #includes
        // 2. Passes it to bindgen::Builder::header
        let bindings = self.make_builder().generate().unwrap().to_string();
        debug!("Bindings: {}", bindings);
        let bindings = syn::parse_str::<ItemMod>(&bindings).unwrap();
        let mut ts = TokenStream2::new();
        bindings.to_tokens(&mut ts);
        ts
    }
}
