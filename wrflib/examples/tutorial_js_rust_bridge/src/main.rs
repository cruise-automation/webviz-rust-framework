// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;

fn sum(values: &[u8]) -> u8 {
    values.iter().sum()
}

fn call_rust(name: String, params: Vec<WrfParam>) -> Vec<WrfParam> {
    if name == "sum" {
        let values = params[0].as_u8_buffer();
        let response = vec![sum(values)].into_param();
        return vec![response];
    }

    vec![]
}

register_call_rust!(call_rust);
